use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::thread;

use libc::{EBADF, EDQUOT, EFAULT, EINTR, EIO, EMFILE, ENOMEM, ENOSPC, EOVERFLOW, c_int, stat as Stat};

use crate::fs::error::{FileCountError, IOError, InterruptError, MetadataOverflowError, IrregularFileError, OOMError, StorageExhaustedError};
use crate::fs::file::{CloneError, CloseError, FileTypeError, MetadataError};
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, Panic, UnexpectedErrorPanic};
use crate::fs::{FileType, Metadata};
use crate::util;

pub(crate) struct Fd(pub c_int);

impl Fd {
    pub fn assert_type_reg(self) -> Result<Fd, FileTypeError> {
        if self.metadata()?.file_type == FileType::Regular {
            Ok(self)
        } else {
            Err(IrregularFileError)?
        }
    }

    pub fn metadata(&self) -> Result<Metadata, MetadataError> {
        let mut raw_meta: MaybeUninit<Stat> = MaybeUninit::uninit();
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
        // SAFETY: close invalidates the provided file descriptor regardless of the outcome, so this
        // method takes ownership of self.
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
        // SAFETY: After this, the file descriptor is invalidated but we are dropping self so it
        // doesn't matter.
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