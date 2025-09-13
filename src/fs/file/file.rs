// TODO: This is only temporary
#![allow(clippy::missing_panics_doc)]

// FIXME: What happens to the CStrings that I've been using indirectly? I bet they aren't dropped.

use std::fmt::{self, Debug, Formatter};
use std::io::RawOsError;
use std::marker::PhantomData;

use libc::{c_int, c_void};

use super::{AccessMode, CloneError, CloseError, LockError, MetadataError, OpenOptions, Read, ReadWrite, SyncError, TryLockError, Write};
use crate::collections::contiguous::Vector;
use crate::fs::panic::{BadFdPanic, InvalidOpPanic, Panic, UnexpectedErrorPanic};
use crate::fs::path::{Abs, Path};
use crate::fs::{Fd, Metadata};
use crate::fs::error::{FileCountError, IOError, InterruptError, LockMemError, OOMError, StorageExhaustedError, SyncUnsupportedError, WouldBlockError};
use crate::util;

pub struct File<Access: AccessMode = ReadWrite> {
    pub(crate) _access: PhantomData<fn() -> Access>,
    pub(crate) fd: Fd,
}

impl<A: AccessMode> File<A> {
    pub fn options() -> OpenOptions<A> {
        OpenOptions::<A>::new()
    }
    
    pub fn metadata(&self) -> Result<Metadata, MetadataError> {
        self.fd.metadata()
    }

    pub fn close(self) -> Result<(), CloseError> {
        self.fd.close()
    }

    pub fn sync(&self) -> Result<(), SyncError> {
        // SAFETY: There is no memory management here and any returned errors are handled.
        if unsafe { libc::fsync(*self.fd) } == -1 {
            match util::fs::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EINTR => Err(InterruptError)?,
                libc::EIO => Err(IOError)?,
                libc::EROFS | libc::EINVAL => Err(SyncUnsupportedError)?,
                libc::ENOSPC | libc::EDQUOT => Err(StorageExhaustedError)?,
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    // TODO: pub fn seek(&self, )

    // TODO: applicable metadata setters

    pub fn try_clone(&self) -> Result<File, CloneError> {
        let new_fd = unsafe { libc::dup(*self.fd) };
        if new_fd == -1 {
            match util::fs::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EMFILE => Err(FileCountError)?,
                libc::ENOMEM => Err(OOMError)?,
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(File {
            _access: PhantomData,
            fd: Fd(new_fd),
        })
    }

    pub(crate) fn flock_raw(&self, flags: c_int) -> Result<(), LockError> {
        if unsafe { libc::flock(*self.fd, flags) } == -1 {
            match util::fs::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EINTR => Err(InterruptError)?,
                libc::EINVAL => InvalidOpPanic.panic(),
                libc::ENOLCK => Err(LockMemError)?,
                // EWOULDBLOCK gets grouped under unexpected.
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    pub fn lock(&self) -> Result<(), LockError> {
        self.flock_raw(libc::LOCK_EX)
    }

    pub fn lock_shared(&self) -> Result<(), LockError> {
        self.flock_raw(libc::LOCK_SH)
    }

    pub(crate) fn try_flock_raw(&self, flags: c_int) -> Result<(), TryLockError> {
        if unsafe { libc::flock(*self.fd, flags | libc::LOCK_NB) } == -1 {
            match util::fs::err_no() {
                libc::EBADF => BadFdPanic.panic(),
                libc::EINTR => Err(InterruptError)?,
                libc::EINVAL => InvalidOpPanic.panic(),
                libc::ENOLCK => Err(LockMemError)?,
                libc::EWOULDBLOCK => Err(WouldBlockError)?,
                e => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        self.try_flock_raw(libc::LOCK_EX)
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        self.try_flock_raw(libc::LOCK_SH)
    }
    

    pub fn unlock(&self) -> Result<(), LockError> {
        self.flock_raw(libc::LOCK_UN)
    }
}

impl File<ReadWrite> {
    pub fn open<P: AsRef<Path<Abs>>>(
        file_path: P,
    ) -> Result<File<ReadWrite>, RawOsError> {
        File::<ReadWrite>::options().open(file_path)
    }

    pub fn create<P: AsRef<Path<Abs>>>(
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, RawOsError> {
        File::<ReadWrite>::options()
            .create_only()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn open_or_create<P: AsRef<Path<Abs>>>(
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, RawOsError> {
        File::<ReadWrite>::options()
            .create_if_absent()
            .mode(file_mode)
            .open(file_path)
    }
}


impl<A: Read> File<A> {
    pub(crate) fn read_raw(&self, buf: *mut c_void, size: usize) -> Result<usize, RawOsError> {
        match unsafe { libc::read(*self.fd, buf, size) } {
            -1 => Err(util::fs::err_no()),
            count => Ok(count as usize),
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, RawOsError> {
        self.read_raw(buf.as_mut_ptr().cast(), buf.len())
    }

    pub fn read_all_vec(&self) -> Result<Vector<u8>, RawOsError> {
        // This doesn't read the terminating byte at the moment.
        let size = self.metadata().unwrap().size as usize; // FIXME
        let buf: Vector<u8> = Vector::with_cap(size);
        let (ptr, len, cap) = buf.into_parts();

        match self.read_raw(unsafe { ptr.as_ptr().add(len).cast() }, cap) {
            Err(err) => {
                unsafe { drop(Vector::from_parts(ptr, len, cap)); }
                Err(err)
            },
            Ok(count) if size > count => todo!("Repeat until all bytes are read!"), // FIXME
            Ok(count) => unsafe { Ok(Vector::from_parts(ptr, len + count, cap)) },
        }
    }

    pub fn read_all_string(&self) -> Result<String, RawOsError> {
        Ok(String::from_utf8(self.read_all_vec()?.into()).unwrap())
    }
}

impl<A: Write> File<A> {
    pub(crate) fn write_raw(&self, buf: *const c_void, size: usize) -> Result<usize, RawOsError> {
        match unsafe { libc::write(*self.fd, buf, size) } {
            -1 => Err(util::fs::err_no()),
            count => Ok(count as usize),
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, RawOsError> {
        self.write_raw(buf.as_ptr().cast(), buf.len())
    }
}

impl<A: AccessMode> Debug for File<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("File")
            .field("<access>", &util::fmt::raw_type_name::<A>())
            .field("fd", &self.fd).finish()
    }
}