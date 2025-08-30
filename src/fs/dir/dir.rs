use std::io::RawOsError;
use std::os::unix::ffi::OsStrExt;

use libc::{O_DIRECTORY, O_PATH, c_char, c_int};

use crate::fs::dir::DirEntries;
use crate::fs::file::CloseError;
use crate::fs::path::{Abs, Path};
use crate::fs::util::{self, Fd};
pub use crate::fs::util::Metadata;

#[derive(Debug)]
pub struct Directory {
    pub(crate) fd: Fd,
}

impl Directory {
    pub fn open<P: AsRef<Path<Abs>>>(dir_path: P) -> Result<Directory, RawOsError> {
        let pathname: *const c_char = dir_path.as_ref().as_os_str().as_bytes().as_ptr().cast();

        let flags: c_int = O_PATH | O_DIRECTORY;

        match unsafe { libc::open(pathname, flags) } {
            -1 => Err(util::err_no()),
            fd => Ok(Directory {
                fd: Fd(fd),
            }),
        }
    }

    pub const fn entries<'a>(&'a self) -> DirEntries<'a> {
        DirEntries {
            dir: self,
            buf: None,
            index: 0,
        }
    }

    // TODO: impl drop for this and don't wrap file.

    pub fn metadata(&self) -> Metadata {
        self.fd.metadata()
    }

    pub fn close(self) -> Result<(), CloseError> {
        self.fd.close()
    }
}
