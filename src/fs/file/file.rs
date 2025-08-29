// TODO: This is only temporary
#![allow(clippy::missing_panics_doc)]

// FIXME: What happens to the CStrings that I've been using indirectly? I bet they aren't dropped.

use std::io::RawOsError;

use libc::c_void;

use super::{CloseError, OpenOptions, SyncError};
use crate::collections::contiguous::Vector;
use crate::fs::path::AbsPath;
use crate::fs::util::{self, Fd};
pub use crate::fs::util::Metadata;
use crate::fs::error::{
    BadFDError, IOError, InterruptError, StorageExhaustedError, SyncUnsupportedError,
    UnexpectedError,
};

#[derive(Debug)]
pub struct File {
    pub(crate) fd: Fd,
}

impl File {
    pub fn open<P: AsRef<AbsPath>>(file_path: P) -> Result<File, RawOsError> {
        File::options().open(file_path)
    }

    pub fn create<P: AsRef<AbsPath>>(file_path: P, file_mode: u16) -> Result<File, RawOsError> {
        File::options()
            .create_only()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn open_or_create<P: AsRef<AbsPath>>(file_path: P, file_mode: u16) -> Result<File, RawOsError> {
        File::options()
            .create_if_absent()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub(crate) fn read_raw(&self, buf: *mut c_void, size: usize) -> Result<usize, RawOsError> {
        match unsafe { libc::read(*self.fd, buf, size) } {
            -1 => Err(util::err_no()),
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
        match unsafe { libc::write(*self.fd, buf, size) } {
            -1 => Err(util::err_no()),
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

    pub fn metadata(&self) -> Metadata {
        self.fd.metadata()
    }

    pub fn close(self) -> Result<(), CloseError> {
        // SAFETY: This method take ownership of self, so fd is not used again.
        self.fd.close()
    }

    pub fn sync(&self) -> Result<(), SyncError> {
        // SAFETY: There is no memory management here and any returned errors are handled.
        if unsafe { libc::fsync(*self.fd) } == -1 {
            match util::err_no() {
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