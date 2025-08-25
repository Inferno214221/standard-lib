use std::ffi::{OsStr, OsString};

use crate::fs::path::{self, OwnedAbsPath, PathLike, sealed::PathInternals};

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
pub struct RelPath {
    pub(crate) inner: OsStr,
}

impl OwnedRelPath {
    pub fn relative_to<'a>(&self, abs: &'a mut OwnedAbsPath) -> &'a mut OwnedAbsPath {
        abs.join(self);
        abs
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
