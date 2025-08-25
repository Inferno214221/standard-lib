use std::borrow::{Borrow, BorrowMut};
use std::env;
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, DerefMut};

use crate::fs::path::{self, OwnedPathLike, PathLike};
use crate::fs::path::abs::DisplayAbs;
use crate::fs::path::sealed::{OwnedPathInternals, PathInternals};

/// TODO
///
/// # Invariants
/// - The string starts with '/'.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0.
pub struct OwnedAbsPath {
    pub(crate) inner: OsString,
}

/// TODO
///
/// # Invariants
/// - The string starts with '/'.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0.
#[repr(transparent)]
pub struct AbsPath {
    pub(crate) inner: OsStr,
}

impl OwnedAbsPath {
    pub fn root() -> OwnedAbsPath {
        OwnedAbsPath {
            inner: OsString::from("/"),
        }
    }

    pub fn home() -> Option<OwnedAbsPath> {
        // TODO: This is a terrible implementation, it copies an owned PathBuf. Also, I'd like to
        // avoid env::home_dir().
        env::home_dir().map(|dir| OwnedAbsPath::from(dir.as_os_str()))
    }

    pub fn cwd() -> Option<OwnedAbsPath> {
        // libc::getcwd()
        todo!()
    }
}

impl OwnedPathInternals for OwnedAbsPath {
    fn inner_mut(&mut self) -> &mut OsString {
        &mut self.inner
    }

    fn inner(&self) -> &OsString {
        &self.inner
    }
    
    unsafe fn new_unchecked(inner: OsString) -> Self {
        OwnedAbsPath {
            inner,
        }
    }
}

impl OwnedPathLike for OwnedAbsPath {}

impl From<&OsStr> for OwnedAbsPath {
    fn from(value: &OsStr) -> Self {
        OwnedAbsPath {
            inner: path::sanitize_os_string(value, b"/"),
        }
    }
}

impl AbsPath {
    pub const unsafe fn new_unchecked(value: &OsStr) -> &AbsPath {
        unsafe { &*(value as *const OsStr as *const AbsPath) }
    }

    pub const unsafe fn new_unchecked_mut(value: &mut OsStr) -> &mut AbsPath {
        unsafe { &mut *(value as *mut OsStr as *mut AbsPath) }
    }

    // pub fn force_relative(&self, from: &AbsPath) {
    //     // TODO: Include ../.. etc.
    //     todo!()
    // }

    pub const fn display<'a>(&'a self) -> DisplayAbs<'a> {
        DisplayAbs {
            inner: self,
        }
    }

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

impl PathInternals for AbsPath {
    fn inner_mut(&mut self) -> &mut OsStr {
        &mut self.inner
    }

    fn inner(&self) -> &OsStr {
        &self.inner
    }
}

impl PathLike for AbsPath {
    type Owned = OwnedAbsPath;
}

impl Deref for OwnedAbsPath {
    type Target = AbsPath;

    fn deref(&self) -> &Self::Target {
        unsafe { AbsPath::new_unchecked(&self.inner) }
    }
}

impl DerefMut for OwnedAbsPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { AbsPath::new_unchecked_mut(&mut self.inner) }
    }
}

impl AsRef<AbsPath> for OwnedAbsPath {
    fn as_ref(&self) -> &AbsPath {
        self.deref()
    }
}

impl AsMut<AbsPath> for OwnedAbsPath {
    fn as_mut(&mut self) -> &mut AbsPath {
        self.deref_mut()
    }
}

impl Borrow<AbsPath> for OwnedAbsPath {
    fn borrow(&self) -> &AbsPath {
        self.as_ref()
    }
}

impl BorrowMut<AbsPath> for OwnedAbsPath {
    fn borrow_mut(&mut self) -> &mut AbsPath {
        self.as_mut()
    }
}

impl AsRef<OsStr> for OwnedAbsPath {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_ref()
    }
}

impl AsRef<OsStr> for AbsPath {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}