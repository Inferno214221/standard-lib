// TODO: This is only temporary
#![allow(clippy::missing_panics_doc)]

// FIXME: What happens to the CStrings that I've been using indirectly? I bet they aren't dropped.

use std::fmt::{self, Debug, Formatter};
use std::io::RawOsError;
use std::marker::PhantomData;

use libc::{EBADF, EDQUOT, EINTR, EINVAL, EIO, ENOLCK, ENOSPC, EROFS, EWOULDBLOCK, LOCK_EX, LOCK_SH, LOCK_UN, c_int, c_void};

use super::{AccessMode, CloneError, CloseError, LockError, MetadataError, OpenOptions, Read, ReadWrite, SyncError, TryLockError, Write};
use crate::collections::contiguous::Vector;
use crate::fs::file::{CreateError, NoCreate, OpenError, TempError};
use crate::fs::panic::{BadFdPanic, InvalidOpPanic, Panic, UnexpectedErrorPanic};
use crate::fs::{Abs, Directory, Fd, Metadata, OwnedPath, Path, Rel};
use crate::fs::error::{IOError, InterruptError, LockMemError, StorageExhaustedError, SyncUnsupportedError, WouldBlockError};
use crate::util;

pub(crate) const DEF_FILE_MODE: c_int = 0o666;

/// An open file, allowing for reading and writing according to the associated [`AccessMode`]. The
/// underlying file is guaranteed to exist for the lifetime of the `File`.
// TODO: More docs here.
pub struct File<Access: AccessMode = ReadWrite> {
    pub(crate) _access: PhantomData<fn() -> Access>,
    pub(crate) fd: Fd,
}

impl File {
    pub fn options() -> OpenOptions<ReadWrite, NoCreate> {
        OpenOptions::<ReadWrite, NoCreate>::new()
    }
}

impl<A: AccessMode> File<A> {
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
                EBADF           => BadFdPanic.panic(),
                EINTR           => Err(InterruptError)?,
                EIO             => Err(IOError)?,
                EROFS | EINVAL  => Err(SyncUnsupportedError)?,
                ENOSPC | EDQUOT => Err(StorageExhaustedError)?,
                e               => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    // TODO: pub fn seek(&self, )

    // TODO: applicable metadata setters

    pub fn try_clone(&self) -> Result<File, CloneError> {
        self.fd.try_clone().map(|new_fd| File {
            _access: PhantomData,
            fd: new_fd,
        })
    }

    pub(crate) fn flock_raw(&self, flags: c_int) -> Result<(), LockError> {
        if unsafe { libc::flock(*self.fd, flags) } == -1 {
            match util::fs::err_no() {
                EBADF  => BadFdPanic.panic(),
                EINTR  => Err(InterruptError)?,
                EINVAL => InvalidOpPanic.panic(),
                ENOLCK => Err(LockMemError)?,
                // EWOULDBLOCK gets grouped under unexpected.
                e      => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    pub fn lock(&self) -> Result<(), LockError> {
        self.flock_raw(LOCK_EX)
    }

    pub fn lock_shared(&self) -> Result<(), LockError> {
        self.flock_raw(LOCK_SH)
    }

    pub(crate) fn try_flock_raw(&self, flags: c_int) -> Result<(), TryLockError> {
        if unsafe { libc::flock(*self.fd, flags | libc::LOCK_NB) } == -1 {
            match util::fs::err_no() {
                EBADF       => BadFdPanic.panic(),
                EINTR       => Err(InterruptError)?,
                EINVAL      => InvalidOpPanic.panic(),
                ENOLCK      => Err(LockMemError)?,
                EWOULDBLOCK => Err(WouldBlockError)?,
                e           => UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        self.try_flock_raw(LOCK_EX)
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        self.try_flock_raw(LOCK_SH)
    }
    

    pub fn unlock(&self) -> Result<(), LockError> {
        self.flock_raw(LOCK_UN)
    }
}

impl File<ReadWrite> {
    pub fn open<P: Into<OwnedPath<Abs>>>(
        file_path: P,
    ) -> Result<File<ReadWrite>, OpenError> {
        File::options()
            .open(file_path)
    }

    pub fn create<P: Into<OwnedPath<Abs>>>(
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, CreateError> {
        File::options()
            .create()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn open_or_create<P: Into<OwnedPath<Abs>>>(
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, OpenError> {
        File::options()
            .create_if_missing()
            .mode(file_mode)
            .open(file_path)
    }

    pub fn open_rel<P: Into<OwnedPath<Rel>>>(
        relative_to: &Directory,
        file_path: P
    ) -> Result<File<ReadWrite>, OpenError> {
        File::options()
            .open_rel(relative_to, file_path)
    }

    pub fn create_rel<P: Into<OwnedPath<Rel>>>(
        relative_to: &Directory,
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, CreateError> {
        File::options()
            .create()
            .mode(file_mode)
            .open_rel(relative_to, file_path)
    }

    pub fn open_or_create_rel<P: Into<OwnedPath<Rel>>>(
        relative_to: &Directory,
        file_path: P,
        file_mode: u16,
    ) -> Result<File<ReadWrite>, OpenError> {
        File::options()
            .create_if_missing()
            .mode(file_mode)
            .open_rel(relative_to, file_path)
    }

    pub fn create_temp() -> Result<File<ReadWrite>, TempError> {
        File::<ReadWrite>::options()
            .create_temp()
            .mode(0o700)
            .open(unsafe { Path::<Abs>::from_unchecked("/tmp") })
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