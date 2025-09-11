use std::ffi::CString;
use std::io::RawOsError;

use libc::{O_DIRECTORY, c_int};

use crate::collections::contiguous::Array;
use crate::fs::dir::DirEntries;
use crate::fs::file::{CloseError, MetadataError};
use crate::fs::path::{Abs, Path};
use crate::fs::util::{self, Fd};
pub use crate::fs::util::Metadata;

use super::BUFFER_SIZE;

#[derive(Debug)]
pub struct Directory {
    pub(crate) fd: Fd,
}

impl Directory {
    pub fn open<P: AsRef<Path<Abs>>>(dir_path: P) -> Result<Directory, RawOsError> {
        let pathname = CString::from(dir_path.as_ref().to_owned());

        let flags: c_int = O_DIRECTORY; // Can't open as O_PATH because we need to read entries.

        match unsafe { libc::open(pathname.as_ptr().cast(), flags) } {
            -1 => Err(util::err_no()),
            fd => Ok(Directory {
                fd: Fd(fd),
            }),
        }
    }

    pub fn entries<'a>(&'a self) -> DirEntries<'a> {
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
}
