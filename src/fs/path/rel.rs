use std::ffi::OsStr;

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