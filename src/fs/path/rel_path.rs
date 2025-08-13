use std::ffi::{OsStr, OsString};

use super::{AbsPath, sealed::PathInternals, PathLike};

/// TODO
/// 
/// # Invariants
/// - The string starts with no "./".
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0 but is followed by exactly one.
pub struct RelPath {
    pub(crate) inner: OsString,
}

impl RelPath {
    pub fn relative_to(&self, abs: &mut AbsPath) {
        abs.join(self)
    }
}

impl PathInternals for RelPath {
    fn inner_mut(&mut self) -> &mut OsString {
        &mut self.inner
    }

    fn inner(&self) -> &OsString {
        &self.inner
    }
}

impl PathLike for RelPath {
    // TODO
}

impl From<&OsStr> for RelPath {
    fn from(value: &OsStr) -> Self {        
        RelPath {
            inner: super::sanitize_os_string(value, b"./"),
        }
    }
}