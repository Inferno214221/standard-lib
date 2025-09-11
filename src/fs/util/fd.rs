use std::mem::MaybeUninit;
use std::ops::Deref;
use std::thread;

use libc::{c_int, stat};

use crate::fs::error::{MetadataOverflowError, IOError, InterruptError, OOMError, StorageExhaustedError};
use crate::fs::file::{CloseError, MetadataError};
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, Panic, UnexpectedErrorPanic};
use crate::fs::util::{self, FileType, Metadata};

#[derive(Debug)]
pub(crate) struct Fd(pub c_int);

impl Fd {
    #[allow(clippy::unnecessary_cast)]
    pub fn metadata(&self) -> Result<Metadata, MetadataError> {
        let mut raw_meta: MaybeUninit<stat> = MaybeUninit::uninit();
        if unsafe { libc::fstat(self.0, raw_meta.as_mut_ptr()) } == -1 {
            match util::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EFAULT => BadStackAddrPanic.panic(),
                libc::ENOMEM => Err(OOMError)?,
                libc::EOVERFLOW => Err(MetadataOverflowError)?,
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        // SAFETY: fstat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Ok(Metadata {
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
        })
    }
    
    pub fn close(self) -> Result<(), CloseError> {
        // SAFETY: close invalidates the provided file descriptor regardless of the outcome, so this
        // method takes ownership of self.
        if unsafe { libc::close(self.0) } == -1 {
            match util::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EINTR => Err(InterruptError)?,
                libc::EIO => Err(IOError)?,
                libc::ENOSPC | libc::EDQUOT => Err(StorageExhaustedError)?,
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }
}

impl Deref for Fd {
    type Target = c_int;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        // SAFETY: After this, the file descriptor is invalidated but we are dropping self so it
        // doesn't matter.
        if unsafe { libc::close(self.0) } == -1
            // Panic only if we aren't already, to prevent aborting an existing unwind.
            && !thread::panicking()
        {
            panic!("error while dropping file descriptor: {}", match util::err_no() {
                libc::EBADF => BadFdPanic.to_string(),
                libc::EINTR => InterruptError.to_string(),
                libc::EIO => IOError.to_string(),
                libc::ENOSPC | libc::EDQUOT => StorageExhaustedError.to_string(),
                e => UnexpectedErrorPanic(e).to_string(),
            });
        }
    }
}