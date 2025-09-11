use std::ffi::{OsStr, OsString};
use std::mem::MaybeUninit;
use std::num::NonZero;
use std::ptr::NonNull;

use crate::collections::contiguous::Array;
use crate::fs::dir::Directory;
use crate::fs::error::RemovedDirectoryError;
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, NotADir, Panic, UnexpectedErrorPanic};
use crate::fs::util::{self, FileType};

#[derive(Debug)]
#[repr(C)]
struct DirEntrySized {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u8,
    pub d_type: u8,
}

// TODO: Add wrapped methods on DirEntry for the ..at syscalls, e.g. fstatat.

#[derive(Debug)]
pub struct DirEntry {
    pub name: OsString,
    pub file_type_hint: Option<FileType>,
    pub inode_num: NonZero<u64>,
    // Neither of these have any relevance to users.
    // pub d_off: i64,
    // pub d_reclen: u8,
}

pub(crate) const BUFFER_SIZE: usize = 1024;

pub struct DirEntries<'a> {
    pub(crate) dir: &'a Directory,
    pub(crate) buf: Array<MaybeUninit<u8>>,
    pub(crate) head: NonNull<MaybeUninit<u8>>,
    pub(crate) rem: usize,
}

// TODO: need to ensure that undersized buffer is handled.

impl<'a> Iterator for DirEntries<'a> {
    type Item = Result<DirEntry, RemovedDirectoryError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            loop {
                match unsafe {
                    util::getdents(*self.dir.fd, self.buf.as_ptr().cast_mut().cast(), self.buf.size)
                } {
                    -1 => match util::err_no() {
                        libc::EBADF => BadFdPanic.panic(),
                        libc::EFAULT => BadStackAddrPanic.panic(),
                        // TODO: Handle (array) overflows etc here? Smart size selection?
                        // If the buffer isn't large enough, double it. Panics in the event of an
                        // overflow.
                        // This is currently the only way that the buffer size can change.
                        libc::EINVAL => self.buf = Array::new_uninit(self.buf.size * 2),
                        // TODO: Do I really have to return this? Result from an Iterator is gross.
                        libc::ENOENT => return Some(Err(RemovedDirectoryError)),
                        libc::ENOTDIR => NotADir.panic(),
                        e => UnexpectedErrorPanic(e).panic(),
                    },
                    0 => None?,
                    count => self.rem = count as usize,
                }
            }
        }
        let sized = unsafe { self.head.cast::<DirEntrySized>().as_ref() };

        let entry_size = sized.d_reclen as usize;

        let entry = if let Ok(inode_num) = NonZero::try_from(sized.d_ino) {
            // name can go from (head + 19)..=(head + entry_size - 2).
            let mut name_vec = Vec::with_capacity(entry_size - 20);
            unsafe {
                let mut char_ptr = self.head.add(19).cast();
                while char_ptr.read() != 0 {
                    name_vec.push(char_ptr.read());
                    char_ptr = char_ptr.add(1);
                }
            }

            let name = unsafe { OsString::from_encoded_bytes_unchecked(name_vec) };

            if name == OsStr::new(".") || name == OsStr::new("..") {
                // Skip redundant "." and ".." entries.
                None
            } else {
                Some(DirEntry {
                    inode_num,
                    file_type_hint: FileType::from_dirent_type(sized.d_type),
                    name,
                })
            }
        } else {
            // Skip entries with inode number zero, these are considered deleted on some file
            // systems and generally invalid on others.
            None
        };

        // head + entry_size - 1: first nul terminator for string.
        // head + entry_size: start of next dirent.
        self.head = unsafe { self.head.add(entry_size) };
        // If head is now outside the bounds of the buffer, there should be no remaining elements
        // and head will be reset next iteration anyway.
        // It turns out this is also was glibc does with readdir.
        self.rem -= entry_size;

        match entry {
            Some(e) => Some(Ok(e)),
            // If a None has made it to this point without propagating immediately, we are skipping
            // a value and need to iterate again.
            None => self.next(),
        }
    }
}