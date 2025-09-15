use std::ffi::CString;
use std::io::RawOsError;

use libc::{O_DIRECTORY, c_int};

use crate::collections::contiguous::Array;
use crate::fs::dir::DirEntries;
use crate::fs::file::{CloneError, CloseError, MetadataError};
use crate::fs::path::{Abs, Path};
use crate::fs::{Fd, Metadata};
use crate::util;

use super::BUFFER_SIZE;

// TODO: Verify that all ..at syscalls support this constant.
// TODO: This might be better placed in an env module or something.
/// A special constant [`Directory`] that always represents to current directory of the process. Can
/// be passed to functions in place of a manually opened `Directory` to indicate that the operation
/// should use the process's cwd.
pub const CWD: Directory = Directory {
    fd: Fd(libc::AT_FDCWD),
};

/// An open directory that is guaranteed to exist for the lifetime of the `Directory`. Can also be
/// used to obtain an iterator over each contained [`DirEntry`](super::DirEntry).
/// 
/// Unlike [`File`](crate::fs::File)s, `Directory`s are currently not associated with an access mode
/// to restrict operations at compile time. This may be changed in the future however, if it helps
/// to better represent the underlying entity and its functionality.
#[derive(Debug)]
pub struct Directory {
    pub(crate) fd: Fd,
}

impl Directory {
    pub fn open<P: AsRef<Path<Abs>>>(dir_path: P) -> Result<Directory, RawOsError> {
        let pathname = CString::from(dir_path.as_ref().to_owned());

        let flags: c_int = O_DIRECTORY; // Can't open as O_PATH because we need to read entries.

        match unsafe { libc::open(pathname.as_ptr().cast(), flags) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(Directory {
                fd: Fd(fd),
            }),
        }
    }

    // pub fn create<P: AsRef<Path<Abs>>>(dir_path: P) -> Result<Directory, RawOsError>;

    // TODO: relative open/create variants.

    pub fn read_entries<'a>(&'a self) -> DirEntries<'a> {
        let buf = Array::new_uninit(BUFFER_SIZE);
        DirEntries {
            dir: self,
            head: buf.ptr,
            buf,
            rem: 0,
        }
    }

    pub fn metadata(&self) -> Result<Metadata, MetadataError> {
        self.fd.metadata()
    }

    pub fn close(self) -> Result<(), CloseError> {
        self.fd.close()
    }

    pub fn try_clone(&self) -> Result<Directory, CloneError> {
        self.fd.try_clone().map(|new_fd| Directory {
            fd: new_fd,
        })
    }
}
