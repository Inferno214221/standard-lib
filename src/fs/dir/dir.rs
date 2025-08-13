use std::io::RawOsError;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use libc::{c_char, c_int, O_DIRECTORY, O_PATH};

use crate::fs::dir::DirEntries;
use crate::fs::file::{CloseError, File, Metadata};
use crate::fs::syscall;

#[derive(Debug)]
pub struct Directory {
    pub(crate) file: File,
}

impl Directory {
    pub fn open(dir_path: &Path) -> Result<Directory, RawOsError> {
        let pathname: *const c_char = dir_path.as_os_str().as_bytes().as_ptr().cast();

        let flags: c_int = O_PATH | O_DIRECTORY;

        match unsafe { libc::open(pathname, flags) } {
            -1 => Err(syscall::err_no()),
            fd => Ok(Directory {
                file: File { fd },
            }),
        }
    }

    pub const fn entries(&self) -> DirEntries {
        DirEntries {
            fd: self.file.fd,
            buf: None,
            index: 0,
        }
    }

    // TODO: impl drop for this and don't wrap file.

    pub fn metadata(&self) -> Metadata {
        self.file.metadata()
    }

    pub fn close(self) -> Result<(), CloseError> {
        self.file.close()
    }
}