use std::env;
use std::ffi::{CString, OsStr, OsString};
use std::io::RawOsError;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use libc::stat as Stat;

use super::{OwnedPath, Path, PathState};
use crate::fs::error::{ExcessiveLinksError, MetadataOverflowError, MissingComponentError, NoSearchError, NonDirComponentError, OOMError, PathLengthError};
use crate::fs::panic::{BadFdPanic, BadStackAddrPanic, Panic, UnexpectedErrorPanic};
use crate::fs::path::{PathError, PathOrMetadataError};
use crate::fs::{Metadata, util};
use crate::fs::file::MetadataError;
use crate::util::sealed::Sealed;

#[derive(Debug)]
pub enum Abs {}

impl Sealed for Abs {}

impl PathState for Abs {}

impl OwnedPath<Abs> {
    pub fn root() -> OwnedPath<Abs> {
        OwnedPath::<Abs> {
            _state: PhantomData,
            inner: OsString::from("/"),
        }
    }

    pub fn home() -> Option<OwnedPath<Abs>> {
        // TODO: This is a terrible implementation, it copies an owned PathBuf. Also, I'd like to
        // avoid env::home_dir().
        env::home_dir().map(|dir| OwnedPath::<Abs>::from(dir.as_os_str()))
    }

    pub fn cwd() -> Option<OwnedPath<Abs>> {
        // libc::getcwd()
        env::current_dir().ok().map(|dir| OwnedPath::<Abs>::from(dir.as_os_str()))
    }
}

impl Path<Abs> {
    pub fn read_all_links(&self) -> Result<OwnedPath<Abs>, RawOsError> {
        todo!("canonicalize with many readlink calls, needs to handle nonexistence")
    }

    pub fn normalize_lexically(&self) -> OwnedPath<Abs> {
        todo!("use components iter and collect")
    }

    // pub fn make_relative(&self, from: &AbsPath) {
    //     // TODO: Include ../.. etc.
    //     todo!()
    // }

    // no follow with O_NOFOLLOW

    // read_* shortcuts for file

    pub(crate) fn match_metadata_error() -> Result<(), PathOrMetadataError> {
        match util::err_no() {
            libc::EACCES => Err(PathError::from(NoSearchError))?,
            libc::EBADF => BadFdPanic.panic(),
            libc::EFAULT => BadStackAddrPanic.panic(),
            libc::ELOOP => Err(PathError::from(ExcessiveLinksError))?,
            libc::ENAMETOOLONG => Err(PathError::from(PathLengthError))?,
            libc::ENOENT => Err(PathError::from(MissingComponentError))?,
            libc::ENOMEM => Err(MetadataError::from(OOMError))?,
            libc::ENOTDIR => Err(PathError::from(NonDirComponentError))?,
            libc::EOVERFLOW => Err(MetadataError::from(MetadataOverflowError))?,
            e => UnexpectedErrorPanic(e).panic(),
        }
    }

    pub fn metadata(&self) -> Result<Metadata, PathOrMetadataError> {
        // FIXME: Copy here feels bad.
        let pathname = CString::from(self.to_owned());

        let mut raw_meta: MaybeUninit<Stat> = MaybeUninit::uninit();
        if unsafe { libc::stat(pathname.as_ptr().cast(), raw_meta.as_mut_ptr()) } == -1 {
            Self::match_metadata_error()?
        }
        // SAFETY: stat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Ok(Metadata::from_stat(raw))
    }

    pub fn metadata_no_follow(&self) -> Result<Metadata, PathOrMetadataError> {
        let pathname = CString::from(self.to_owned());

        let mut raw_meta: MaybeUninit<Stat> = MaybeUninit::uninit();
        if unsafe { libc::lstat(pathname.as_ptr().cast(), raw_meta.as_mut_ptr()) } == -1 {
            Self::match_metadata_error()?
        }
        // SAFETY: stat either initializes raw_meta or returns an error and diverges.
        let raw = unsafe { raw_meta.assume_init() };

        Ok(Metadata::from_stat(raw))
    }

    // NOTE: Symlinks can't be opened, so all symlink-related APIs need to be handled here.

    // create_symlink
    // read_link
    // is_symlink
    // create_hardlink

    // rename
    // move_dir
    // remove (unlink)
    // rmdir
    // copy (sendfile)
    // chmod
    // chown
    // exists
    // try_exists
    // access

    // set_cwd
}

impl<O: AsRef<OsStr>> From<O> for OwnedPath<Abs> {
    fn from(value: O) -> Self {
        Self::from_os_str_sanitized(value.as_ref())
    }
}
