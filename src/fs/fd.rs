use std::ffi::CString;
use std::fmt::{self, Debug, Formatter};
use std::io::RawOsError;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::thread;

use libc::{EBADF, EDQUOT, EFAULT, EINTR, EIO, EMFILE, ENOMEM, ENOSPC, EOVERFLOW, c_int, stat as Stat};

use crate::fs::error::{FileCountError, IOError, InterruptError, MetadataOverflowError, IncorrectTypeError, OOMError, StorageExhaustedError};
use crate::fs::file::{CloneError, CloseError, FileTypeError, MetadataError};
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, Panic, UnexpectedErrorPanic};
use crate::fs::{Abs, Directory, FileType, Metadata, OwnedPath, Rel};
use crate::util;

pub(crate) struct Fd(pub c_int);

impl Fd {
    pub fn open<P: Into<OwnedPath<Abs>>>(file_path: P, flags: c_int, mode: c_int) -> Result<Fd, RawOsError> {
        let pathname = CString::from(file_path.into());
        // TODO: Permission builder of some type?

        // SAFETY: The pointer obtained from CString::as_ptr() is valid for the lifetime of `pathname`.
        // CString guarantees the data is null-terminated and contains no interior null bytes, which
        // satisfies libc::open's requirements for a valid C string pathname.
        match unsafe { libc::open(pathname.as_ptr().cast(), flags, mode) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(Fd(fd)),
        }
    }

    pub fn open_rel<P: Into<OwnedPath<Rel>>>(
        relative_to: &Directory,
        file_path: P,
        flags: c_int,
        mode: c_int
    ) -> Result<Fd, RawOsError> {
        let pathname = CString::from(file_path.into());

        // SAFETY:
        // - The directory file descriptor (*relative_to.fd) is guaranteed valid because Directory
        //   maintains ownership of a valid, open file descriptor.
        // - pathname.as_ptr().add(1) is safe because Path<Rel> guarantees at least 1 byte (the
        //   leading '/'), and CString adds a null terminator, so .add(1) points to either the next
        //   path component or the null terminator. This skips the leading '/' so openat treats the
        //   path as relative to the directory fd.
        // - The resulting pointer is still valid, null-terminated, and has no interior null bytes.
        match unsafe { libc::openat(
            *relative_to.fd,
            // Skip the leading '/' so that the path is considered relative by the OS.
            pathname.as_ptr().add(1).cast(),
            flags,
            mode
        ) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(Fd(fd)),
        }
    }

    #[inline(always)]
    pub fn assert_type(self, file_type: FileType) -> Result<Fd, FileTypeError> {
        if self.metadata()?.file_type == file_type {
            Ok(self)
        } else {
            Err(IncorrectTypeError)?
        }
    }

    pub fn metadata(&self) -> Result<Metadata, MetadataError> {
        let mut raw_meta: MaybeUninit<Stat> = MaybeUninit::uninit();
        // SAFETY:
        // - self.0 is a valid, open file descriptor (guaranteed by Fd's ownership and invariants).
        // - raw_meta.as_mut_ptr() points to valid, properly aligned stack memory allocated for a
        //   Stat structure. MaybeUninit allows passing uninitialized memory to fstat, which will
        //   initialize it.
        if unsafe { libc::fstat(self.0, raw_meta.as_mut_ptr()) } == -1 {
            match util::fs::err_no() {
                EBADF =>     BadFdPanic.panic(),
                EFAULT =>    BadStackAddrPanic.panic(),
                ENOMEM =>    Err(OOMError)?,
                EOVERFLOW => Err(MetadataOverflowError)?,
                e =>         UnexpectedErrorPanic(e).panic(),
            }
        }
        // SAFETY: fstat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Ok(Metadata::from_stat(raw))
    }
    
    pub fn close(self) -> Result<(), CloseError> {
        // SAFETY:
        // - self.0 is a valid, open file descriptor (guaranteed by Fd's ownership and invariants).
        // - close invalidates the provided file descriptor regardless of the outcome, so this
        //   method takes ownership of self, ensuring the fd cannot be used after this call.
        if unsafe { libc::close(self.0) } == -1 {
            match util::fs::err_no() {
                EBADF =>           BadFdPanic.panic(),
                EINTR =>           Err(InterruptError)?,
                EIO =>             Err(IOError)?,
                ENOSPC | EDQUOT => Err(StorageExhaustedError)?,
                e =>               UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(())
    }

    pub fn try_clone(&self) -> Result<Fd, CloneError> {
        // SAFETY: self.0 is a valid, open file descriptor (guaranteed by Fd's ownership and
        // invariants). dup creates a new file descriptor that refers to the same open file
        // description, or returns -1 on error.
        let new_fd = unsafe { libc::dup(self.0) };
        if new_fd == -1 {
            match util::fs::err_no() {
                EBADF =>  BadFdPanic.panic(),
                EMFILE => Err(FileCountError)?,
                ENOMEM => Err(OOMError)?,
                e =>      UnexpectedErrorPanic(e).panic(),
            }
        }
        Ok(Fd(new_fd))
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
        // SAFETY:
        // - self.0 is a valid, open file descriptor (guaranteed by Fd's ownership and invariants).
        // - After this call, the file descriptor is invalidated, but we are dropping self so it
        //   cannot be used again.
        if unsafe { libc::close(self.0) } == -1
            // Panic only if we aren't already, to prevent aborting an existing unwind.
            && !thread::panicking()
        {
            panic!("error while dropping file descriptor: {}", match util::fs::err_no() {
                EBADF =>           BadFdPanic.to_string(),
                EINTR =>           InterruptError.to_string(),
                EIO =>             IOError.to_string(),
                ENOSPC | EDQUOT => StorageExhaustedError.to_string(),
                e =>               UnexpectedErrorPanic(e).to_string(),
            });
        }
    }
}

impl Debug for Fd {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fd({})", self.0)
    }
}