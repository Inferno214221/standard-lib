// TODO: This is only temporary
#![allow(clippy::missing_panics_doc)]

// FIXME: What happens to the CStrings that I've been using indirectly? I bet they aren't dropped.

use std::io::RawOsError;
use std::mem::MaybeUninit;
use std::path::Path;
use std::thread;

use libc::{c_void, stat};

use super::{CloseError, OpenOptions, SyncError};
use crate::collections::contiguous::Vector;
use crate::fs::syscall;
use crate::fs::{
    BadFDError, BadStackAddrError, Fd, FileMetaOverflowError, IOError, InterruptError, OOMError,
    StorageExhaustedError, SyncUnsupportedError, UnexpectedError,
};

#[derive(Debug)]
pub struct File {
    pub(crate) fd: Fd,
}

// TODO: Ensure that only Files are supported? Prefereably while keeping Metadata lazy - Metadata
// has a size of 120 bytes.
#[derive(Debug)]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory,
    FIFO,
    Symlink,
    Regular,
    Socket,
    Unknown,
}

pub struct Metadata {
    pub size: i64,             // st_size
    pub file_type: FileType,   // st_mode
    pub mode: u16,             // st_mode
    pub uid: u32,              // st_uid
    pub gid: u32,              // st_gid
    pub parent_device_id: u64, // st_dev
    pub self_device_id: u64,   // st_rdev
    // x86_64:
    pub time_accessed: (i64, i64), // st_atime, st_atime_nsec
    pub time_modified: (i64, i64), // st_mtime, st_mtime_nsec
    pub time_changed: (i64, i64),  // st_ctime, st_ctime_nsec
    pub links: u64,                // st_nlink
    pub block_size: i64,           // st_blksize
    // 64-bit:
    pub blocks: i64,    // st_blocks
    pub inode_num: u64, // st_ino
}

impl File {
    pub fn open(file_path: &Path) -> Result<File, RawOsError> {
        File::options().open(file_path)
    }

    pub fn create(file_path: &Path, file_mode: u16) -> Result<File, RawOsError> {
        File::options()
            .create_only()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn open_or_create(file_path: &Path, file_mode: u16) -> Result<File, RawOsError> {
        File::options()
            .create_if_absent()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub(crate) fn read_raw(&self, buf: *mut c_void, size: usize) -> Result<usize, RawOsError> {
        match unsafe { libc::read(self.fd, buf, size) } {
            -1 => Err(syscall::err_no()),
            count => Ok(count as usize),
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, RawOsError> {
        self.read_raw(buf.as_mut_ptr().cast(), buf.len())
    }

    pub fn read_all_vec(&self) -> Result<Vector<u8>, RawOsError> {
        // This doesn't read the terminating byte at the moment.
        let size = self.metadata().size as usize;
        let buf: Vector<u8> = Vector::with_cap(size);
        let (ptr, len, cap) = buf.into_parts();

        match self.read_raw(unsafe { ptr.as_ptr().add(len).cast() }, cap) {
            Err(err) => {
                unsafe { drop(Vector::from_parts(ptr, len, cap)); }
                Err(err)
            },
            Ok(count) if size > count => todo!("Repeat until all bytes are read!"), // TODO
            Ok(count) => unsafe { Ok(Vector::from_parts(ptr, len + count, cap)) },
        }
    }

    pub fn read_all_string(&self) -> Result<String, RawOsError> {
        Ok(String::from_utf8(self.read_all_vec()?.into()).unwrap())
    }

    pub(crate) fn write_raw(&self, buf: *const c_void, size: usize) -> Result<usize, RawOsError> {
        match unsafe { libc::write(self.fd, buf, size) } {
            -1 => Err(syscall::err_no()),
            count => Ok(count as usize),
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, RawOsError> {
        self.write_raw(buf.as_ptr().cast(), buf.len())
    }

    // TODO: pub fn seek(&self, )

    // TODO: pub fn lock(&self) -> Result<(), RawOsError> {}

    // TODO: pub fn lock_shared(&self) -> Result<(), RawOsError> {}

    // TODO: pub fn try_lock(&self) -> Result<(), RawOsError> {}

    // TODO: pub fn try_lock_shared(&self) -> Result<(), RawOsError> {}

    // TODO: pub fn unlock(&self) -> Result<(), RawOsError> {}

    // TODO: pub fn try_clone(&self) -> Result<File, RawOsError> {}

    // TODO: applicable metadata setters

    #[allow(clippy::unnecessary_cast)]
    pub fn metadata(&self) -> Metadata {
        let mut raw_meta: MaybeUninit<stat> = MaybeUninit::uninit();
        if unsafe { libc::fstat(self.fd, raw_meta.as_mut_ptr()) } == -1 {
            match syscall::err_no() {
                libc::EBADF => panic!("{}", BadFDError),
                libc::EFAULT => panic!("{}", BadStackAddrError),
                libc::ENOMEM => panic!("{}", OOMError),
                libc::EOVERFLOW => panic!("{}", FileMetaOverflowError),
                e => panic!("{}", UnexpectedError(e)),
            }
        }
        // SAFETY: fstat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Metadata {
            size: raw.st_size,
            file_type: match raw.st_mode & libc::S_IFMT {
                libc::S_IFBLK => FileType::BlockDevice,
                libc::S_IFCHR => FileType::CharDevice,
                libc::S_IFDIR => FileType::Directory,
                libc::S_IFIFO => FileType::FIFO,
                libc::S_IFLNK => FileType::Symlink,
                libc::S_IFREG => FileType::Regular,
                libc::S_IFSOCK => FileType::Socket,
                _ => FileType::Unknown,
            },
            mode: raw.st_mode as u16,
            uid: raw.st_uid,
            gid: raw.st_gid,
            parent_device_id: raw.st_dev,
            self_device_id: raw.st_rdev,
            time_accessed: (raw.st_atime as i64, raw.st_atime_nsec),
            time_modified: (raw.st_mtime as i64, raw.st_mtime_nsec),
            time_changed: (raw.st_ctime as i64, raw.st_ctime_nsec),
            links: raw.st_nlink as u64,
            block_size: raw.st_blksize as i64,
            blocks: raw.st_blocks as i64,
            inode_num: raw.st_ino as u64,
        }
    }

    pub fn close(self) -> Result<(), CloseError> {
        // SAFETY: close invalidates the provided file descriptor regardless of the outcome, so this
        // method takes ownership of self.
        if unsafe { libc::close(self.fd) } == -1 {
            match syscall::err_no() {
                libc::EBADF => panic!("{}", BadFDError),
                libc::EINTR => Err(InterruptError)?,
                libc::EIO => Err(IOError)?,
                libc::ENOSPC | libc::EDQUOT => Err(StorageExhaustedError)?,
                e => panic!("{}", UnexpectedError(e)),
            }
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<(), SyncError> {
        // SAFETY: There is no memory management here and any returned errors are handled.
        if unsafe { libc::fsync(self.fd) } == -1 {
            match syscall::err_no() {
                libc::EBADF => panic!("{}", BadFDError),
                libc::EINTR => Err(InterruptError)?,
                libc::EIO => Err(IOError)?,
                libc::EROFS | libc::EINVAL => Err(SyncUnsupportedError)?,
                libc::ENOSPC | libc::EDQUOT => Err(StorageExhaustedError)?,
                e => panic!("{}", UnexpectedError(e)),
            }
        }
        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: After this, the file descriptor is invalidated but we are dropping self so it
        // doesn't matter.
        if unsafe { libc::close(self.fd) } == -1
            // Panic only if we aren't already, to prevent aborting an existing unwind.
            && !thread::panicking()
        {
            let error = match syscall::err_no() {
                libc::EBADF => BadFDError.to_string(),
                libc::EINTR => InterruptError.to_string(),
                libc::EIO => IOError.to_string(),
                libc::ENOSPC | libc::EDQUOT => StorageExhaustedError.to_string(),
                e => UnexpectedError(e).to_string(),
            };
            panic!("error while dropping file: {error}");
        }
    }
}