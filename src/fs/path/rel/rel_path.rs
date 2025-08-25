use std::{borrow::{Borrow, BorrowMut}, ffi::{OsStr, OsString}, ops::{Deref, DerefMut}};

use crate::fs::path::{self, rel::DisplayRel, sealed::PathInternals, OwnedAbsPath, PathLike};

/// TODO
///
/// # Invariants
/// - The string starts with '/' internally.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0.
pub struct OwnedRelPath {
    pub(crate) inner: OsString,
}

/// TODO
///
/// # Invariants
/// - The string starts with '/' internally.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0. 
#[repr(transparent)]
pub struct RelPath {
    pub(crate) inner: OsStr,
}

impl OwnedRelPath {
    pub fn resolve<'a>(&self, target: &'a mut OwnedAbsPath) -> &'a mut OwnedAbsPath {
        target.join(self);
        target
    }
}

impl PathInternals for OwnedRelPath {
    fn inner_mut(&mut self) -> &mut OsString {
        &mut self.inner
    }

    fn inner(&self) -> &OsString {
        &self.inner
    }
}

impl PathLike for OwnedRelPath {
    // TODO
}

impl From<&OsStr> for OwnedRelPath {
    fn from(value: &OsStr) -> Self {
        OwnedRelPath {
            inner: path::sanitize_os_string(value, b"/"),
        }
    }
}

impl RelPath {
    pub const unsafe fn new_unchecked(value: &OsStr) -> &RelPath {
        unsafe { &*(value as *const OsStr as *const RelPath) }
    }

    pub const unsafe fn new_unchecked_mut(value: &mut OsStr) -> &mut RelPath {
        unsafe { &mut *(value as *mut OsStr as *mut RelPath) }
    }

    pub const fn display<'a>(&'a self) -> DisplayRel<'a> {
        DisplayRel {
            inner: self,
        }
    }
}

impl Deref for OwnedRelPath {
    type Target = RelPath;

    fn deref(&self) -> &Self::Target {
        unsafe { RelPath::new_unchecked(&*self.inner) }
    }
}

impl DerefMut for OwnedRelPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { RelPath::new_unchecked_mut(&mut *self.inner) }
    }
}

impl AsRef<RelPath> for OwnedRelPath {
    fn as_ref(&self) -> &RelPath {
        self.deref()
    }
}

impl AsMut<RelPath> for OwnedRelPath {
    fn as_mut(&mut self) -> &mut RelPath {
        self.deref_mut()
    }
}

impl Borrow<RelPath> for OwnedRelPath {
    fn borrow(&self) -> &RelPath {
        self.as_ref()
    }
}

impl BorrowMut<RelPath> for OwnedRelPath {
    fn borrow_mut(&mut self) -> &mut RelPath {
        self.as_mut()
    }
}

impl AsRef<OsStr> for OwnedRelPath {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_ref()
    }
}

impl AsRef<OsStr> for RelPath {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}