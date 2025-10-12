use std::ffi::{CString, OsStr, OsString};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use libc::{EACCES, EBADF, EFAULT, EINVAL, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EOVERFLOW, c_int, stat as Stat};

use super::{Abs, OwnedPath, Path, PathState};
use crate::fs::error::{ExcessiveLinksError, MetadataOverflowError, MissingComponentError, NoSearchError, NonDirComponentError, OOMError, PathLengthError};
use crate::fs::file::MetadataError;
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, InvalidOpPanic, Panic, UnexpectedErrorPanic};
use crate::fs::{Directory, Metadata};
use crate::fs::path::{PathError, PathOrMetadataError};
use crate::util::{self, sealed::Sealed};

#[derive(Debug)]
pub enum Rel {}

impl Sealed for Rel {}

impl PathState for Rel {}

impl OwnedPath<Rel> {
    pub fn dot_slash_dot() -> OwnedPath<Rel> {
        OwnedPath::<Rel> {
            _state: PhantomData,
            inner: OsString::from("/."),
        }
    }

    pub fn resolve_root(self) -> OwnedPath<Abs> {
        let OwnedPath { _state, inner } = self;
        OwnedPath {
            _state: PhantomData,
            inner
        }
    }
}

impl Path<Rel> {
    pub fn resolve(&self, mut target: OwnedPath<Abs>) -> OwnedPath<Abs> {
        target.push(self);
        target
    }

    pub fn resolve_root(&self) -> OwnedPath<Abs> {
        self.resolve(OwnedPath::root())
    }

    pub fn resolve_home(&self) -> Option<OwnedPath<Abs>> {
        Some(self.resolve(OwnedPath::home()?))
    }

    pub fn resolve_cwd(&self) -> Option<OwnedPath<Abs>> {
        Some(self.resolve(OwnedPath::cwd()?))
    }

    // TODO: add relative methods which use the ..at syscalls and take a directory.

    pub(crate) fn metadata_raw(&self, relative_to: Directory, flags: c_int) -> Result<Metadata, PathOrMetadataError> {
        // FIXME: Copy here feels bad.
        let pathname = CString::from(self.to_owned());

        let mut raw_meta: MaybeUninit<Stat> = MaybeUninit::uninit();
        if unsafe { libc::fstatat(
            *relative_to.fd,
            // Skip the leading '/' so that the path is considered relative.
            pathname.as_ptr().add(1).cast(),
            raw_meta.as_mut_ptr(),
            flags
        ) } == -1 {
            match util::fs::err_no() {
                EACCES       => Err(PathError::from(NoSearchError))?,
                EBADF        => BadFdPanic.panic(),
                EFAULT       => BadStackAddrPanic.panic(),
                EINVAL       => InvalidOpPanic.panic(),
                ELOOP        => Err(PathError::from(ExcessiveLinksError))?,
                ENAMETOOLONG => Err(PathError::from(PathLengthError))?,
                ENOENT       => Err(PathError::from(MissingComponentError))?,
                ENOMEM       => Err(MetadataError::from(OOMError))?,
                ENOTDIR      => Err(PathError::from(NonDirComponentError))?,
                EOVERFLOW    => Err(MetadataError::from(MetadataOverflowError))?,
                e            => UnexpectedErrorPanic(e).panic(),
            }
        }
        // SAFETY: stat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Ok(Metadata::from_stat(raw))
    }

    pub fn metadata(&self, relative_to: Directory) -> Result<Metadata, PathOrMetadataError> {
        self.metadata_raw(relative_to, 0)
    }

    pub fn metadata_no_follow(&self, relative_to: Directory) -> Result<Metadata, PathOrMetadataError> {
        self.metadata_raw(relative_to, libc::AT_SYMLINK_NOFOLLOW)
    }
}

impl<O: AsRef<OsStr>> From<O> for OwnedPath<Rel> {
    fn from(value: O) -> Self {
        Self::from_os_str_sanitized(value.as_ref())
    }
}
