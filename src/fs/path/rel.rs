use std::{ffi::OsStr, mem};

use super::{Abs, OwnedPath, Path, PathState};
use crate::util::sealed::Sealed;

#[derive(Debug)]
pub enum Rel {}

impl Sealed for Rel {}

impl PathState for Rel {}

impl OwnedPath<Rel> {
    pub fn resolve_root(self) -> OwnedPath<Abs> {
        // SAFETY: OwnedPath<Rel> has the same layout as OwnedPath<Abs> and represents the same
        // result as resolving relative to the root.
        unsafe { mem::transmute(self) }
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
}

impl<O: AsRef<OsStr>> From<O> for OwnedPath<Rel> {
    fn from(value: O) -> Self {
        Self::from_os_str_sanitized(value.as_ref())
    }
}
