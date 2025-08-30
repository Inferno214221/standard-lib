use std::borrow::{Borrow, BorrowMut};
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};

use super::{Abs, OwnedPath, Path, sealed};

pub enum Rel {}

impl sealed::PathState for Rel {}

impl OwnedPath<Rel> {
    pub fn resolve<P: AsRef<Path<Abs>>>(&self, target: P) -> OwnedPath<Abs> {
        target.as_ref().join(self)
    }
}

impl Path<Rel> {
    //
}

impl From<&OsStr> for OwnedPath<Rel> {
    fn from(value: &OsStr) -> Self {
        Self::from_os_str_sanitized(value)
    }
}

impl From<&str> for OwnedPath<Rel> {
    fn from(value: &str) -> Self {
        OsStr::new(value).into()
    }
}

impl Deref for OwnedPath<Rel> {
    type Target = Path<Rel>;

    fn deref(&self) -> &Self::Target {
        unsafe { Path::<Rel>::new_unchecked(&self.inner) }
    }
}

impl DerefMut for OwnedPath<Rel> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Path::<Rel>::new_unchecked_mut(&mut self.inner) }
    }
}

impl AsRef<Path<Rel>> for OwnedPath<Rel> {
    fn as_ref(&self) -> &Path<Rel> {
        self.deref()
    }
}

impl AsMut<Path<Rel>> for OwnedPath<Rel> {
    fn as_mut(&mut self) -> &mut Path<Rel> {
        self.deref_mut()
    }
}

impl Borrow<Path<Rel>> for OwnedPath<Rel> {
    fn borrow(&self) -> &Path<Rel> {
        self.as_ref()
    }
}

impl BorrowMut<Path<Rel>> for OwnedPath<Rel> {
    fn borrow_mut(&mut self) -> &mut Path<Rel> {
        self.as_mut()
    }
}

impl AsRef<OsStr> for OwnedPath<Rel> {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_ref()
    }
}

impl AsRef<OsStr> for Path<Rel> {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}