use std::ffi::OsStr;
use std::io::RawOsError;
use std::mem::MaybeUninit;
use std::num::NonZero;
use std::os::unix::ffi::OsStrExt;
use std::ptr::NonNull;
use std::slice;

use crate::collections::contiguous::Array;
use crate::fs::dir::Directory;
use crate::fs::error::RemovedDirectoryError;
use crate::fs::file::ReadWrite;
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, NotADirPanic, Panic, UnexpectedErrorPanic};
use crate::fs::{File, FileType, OwnedPath, Rel};
use crate::util;

#[derive(Debug)]
#[repr(C)]
pub(crate) struct DirEntrySized {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u8,
    pub d_type: u8,
}

// TODO: Add wrapped methods on DirEntry for the ..at syscalls, e.g. fstatat.

/// An entry in a [`Directory`]'s records, taking the form of a reference to the parent `Directory`
/// and the relative path to this entry, along with a few other pieces of information.
/// 
/// Among these other pieces of information, is the entry's inode number and a hint about the
/// [`FileType`] of the entry. Unfortunately, this field is often not present, and therefore
/// considered only a hint. If the `FileType` is present and has some value, it can be trusted
/// (subject to TOCTOU restrictions), but the possibility of it not existing should always be
/// considered or even expected.
#[derive(Debug)]
pub struct DirEntry<'a> {
    pub parent: &'a Directory,
    pub path: OwnedPath<Rel>,
    pub file_type_hint: Option<FileType>,
    pub inode_num: NonZero<u64>,
    // Neither of these have any relevance to users.
    // pub d_off: i64,
    // pub d_reclen: u8,
}

impl<'a> DirEntry<'a> {
    pub fn name(&self) -> &OsStr {
        self.path.as_os_str_no_lead()
    }

    pub fn open_file(&self) -> Result<File<ReadWrite>, RawOsError> {
        File::options().open_dir_entry(self)
    }

    // pub fn open_dir(&self) -> Result<Directory, _>; // openat
    
    // TODO: forward to path methods
}

pub(crate) const BUFFER_SIZE: usize = 1024;

/// An iterator over a [`Directory`]'s contained entries. Obtainable via
/// [`Directory::read_entries`], this buffered iterator produces [`DirEntry`]s for the referenced
/// directory.
/// 
/// When reading from the file system, the redundant "." and ".." entries are skipped, along with
/// any deleted entries.
// TODO: Heaps of TOCTOU notes.
pub struct DirEntries<'a> {
    pub(crate) dir: &'a Directory,
    pub(crate) buf: Array<MaybeUninit<u8>>,
    pub(crate) head: NonNull<MaybeUninit<u8>>,
    pub(crate) rem: usize,
}

// TODO: need to ensure that undersized buffer is handled.

impl<'a> Iterator for DirEntries<'a> {
    type Item = Result<DirEntry<'a>, RemovedDirectoryError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            loop {
                match unsafe { util::fs::getdents(
                    *self.dir.fd,
                    self.buf.as_ptr().cast_mut().cast(),
                    self.buf.size()
                ) } {
                    -1 => match util::fs::err_no() {
                        libc::EBADF => BadFdPanic.panic(),
                        libc::EFAULT => BadStackAddrPanic.panic(),
                        // TODO: Handle (array) overflows etc here? Smart size selection?
                        // If the buffer isn't large enough, double it. Panics in the event of an
                        // overflow.
                        // This is currently the only way that the buffer size can change.
                        libc::EINVAL => {
                            self.buf = Array::new_uninit(self.buf.size * 2);
                            continue;
                        },
                        // TODO: Do I really have to return this? Result from an Iterator is gross.
                        libc::ENOENT => return Some(Err(RemovedDirectoryError)),
                        libc::ENOTDIR => NotADirPanic.panic(),
                        e => UnexpectedErrorPanic(e).panic(),
                    },
                    0 => None?,
                    count => self.rem = count as usize,
                }
                break;
            }
        }
        let sized = unsafe { self.head.cast::<DirEntrySized>().as_ref() };

        let entry_size = sized.d_reclen as usize;

        let entry = if let Ok(inode_num) = NonZero::try_from(sized.d_ino) {
            let char_head = unsafe { self.head.add(19).cast() };
            let mut char_tail = char_head;
            while unsafe { char_tail.read() } != 0 {
                char_tail = unsafe { char_tail.add(1) };
            }

            let name = unsafe { OwnedPath::<Rel>::from_os_str_sanitized(OsStr::from_bytes(
                slice::from_ptr_range(char_head.as_ptr()..char_tail.as_ptr())
            )) };

            if name.as_os_str() == OsStr::new("/") || name.as_os_str() == OsStr::new("/..") {
                // Skip redundant "." and ".." entries. "." will be normalized to "/", which we
                // don't want either.
                None
            } else {
                Some(DirEntry {
                    parent: self.dir,
                    inode_num,
                    file_type_hint: FileType::from_dirent_type(sized.d_type),
                    path: name,
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