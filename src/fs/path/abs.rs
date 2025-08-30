use std::env;
use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;

use super::{OwnedPath, Path, sealed::PathState};

pub enum Abs {}
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
    // pub fn force_relative(&self, from: &AbsPath) {
    //     // TODO: Include ../.. etc.
    //     todo!()
    // }

    // fn metadata(&self) -> Result<Metadata>;

    // fn open(&self) -> Union(File, Dir, etc.) Union should hold metadata too?

    // no follow with O_NOFOLLOW
    // to open as a specific type, use File::open or Dir::open

    // fn canonicalize

    // fn exists/try_exists

    // fn read_* shortcuts

    // NOTE: Symlinks can't be opened, so all symlink-related APIs need to be handled here.

    // fn is_symlink

    // fn symlink_metadata

    // fn read_link

    // type agnostic methods, e.g. copy, move, rename, etc. chown, chmod?
}

impl From<&OsStr> for OwnedPath<Abs> {
    fn from(value: &OsStr) -> Self {
        Self::from_os_str_sanitized(value)
    }
}

impl From<&str> for OwnedPath<Abs> {
    fn from(value: &str) -> Self {
        OsStr::new(value).into()
    }
}