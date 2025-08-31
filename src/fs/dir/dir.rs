use std::ffi::CString;
use std::io::RawOsError;

use libc::{O_DIRECTORY, O_PATH, c_int};

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
        let pathname = CString::from(dir_path.as_ref().to_owned());

        let flags: c_int = O_PATH | O_DIRECTORY;

        match unsafe { libc::open(pathname.as_ptr().cast(), flags) } {
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
