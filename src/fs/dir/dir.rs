use std::ffi::CString;
use std::io::RawOsError;

use libc::{O_DIRECTORY, c_int, mode_t};

use crate::collections::contiguous::Array;
use crate::fs::dir::{DirEntries, DirEntry};
#[doc(inline)]
pub use crate::fs::file::{CloneError, CloseError, MetadataError, OpenError};
use crate::fs::{Abs, Fd, FileType, Metadata, Path, Rel};
use crate::util;

use super::BUFFER_SIZE;

pub(crate) const DEF_DIR_MODE: c_int = 0o777;

// TODO: This might be better placed in an env module or something.
/// A special constant [`Directory`] that always represents to current directory of the process. Can
/// be passed to functions in place of a manually opened `Directory` to indicate that the operation
/// should use the process's cwd.
pub const CWD: Directory = Directory {
    fd: Fd(libc::AT_FDCWD),
};

/// An open directory that is guaranteed to exist for the lifetime of the `Directory`. Can also be
/// used to obtain an iterator over each contained [`DirEntry`](super::DirEntry).
#[derive(Debug)]
pub struct Directory {
    pub(crate) fd: Fd,
}

impl Directory {
    // TODO: Fix dir_path, file_path naming scheme to path and parent_path
    pub fn open<P: AsRef<Path<Abs>>>(dir_path: P) -> Result<Directory, OpenError> {
        match Fd::open(dir_path, O_DIRECTORY, DEF_DIR_MODE) {
            Ok(fd) => Ok(Directory {
                fd: fd.assert_type(FileType::Directory)?,
            }),
            Err(e) => Err(OpenError::interpret_raw_error(e)),
        }
    }

    pub fn open_rel<P: AsRef<Path<Rel>>>(
        &self,
        relative_to: &Directory,
        dir_path: P
    ) -> Result<Directory, OpenError> {
        match Fd::open_rel(relative_to, dir_path, O_DIRECTORY, DEF_DIR_MODE) {
            Ok(fd) => Ok(Directory {
                fd: fd.assert_type(FileType::Directory)?,
            }),
            Err(e) => Err(OpenError::interpret_raw_error(e)),
        }
    }

    pub fn open_dir_entry(&self, dir_ent: &DirEntry) -> Result<Directory, OpenError> {
        self.open_rel(dir_ent.parent, &dir_ent.path)
    }

    pub fn create<P: AsRef<Path<Abs>>>(
        dir_path: P,
        file_mode: u16
    ) -> Result<Directory, RawOsError> {
        let pathname = CString::from(dir_path.as_ref().to_owned());

        match unsafe { libc::mkdir(pathname.as_ptr().cast(), file_mode as mode_t) } {
            -1 => Err(util::fs::err_no()), // TODO: interpret raw error
            fd => Ok(Directory {
                fd: Fd(fd),
            }),
        }
    }

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
