use std::ffi::{OsStr, OsString};

use crate::fs::path::{PathLike, sealed::PathInternals};

/// TODO
///
/// # Invariants
/// - The string starts with '/'.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0 but is followed by exactly one.
pub struct AbsPath {
    pub(crate) inner: OsString,
}

impl AbsPath {
    pub fn root() -> AbsPath {
        AbsPath {
            inner: OsString::from("/"),
        }
    }

    // TODO: pub fn home() -> Option<AbsPath>; // Should this be an env thing?

    pub fn cwd() -> Option<AbsPath> {
        // libc::getcwd()
        todo!()
    }
}

impl PathInternals for AbsPath {
    fn inner_mut(&mut self) -> &mut OsString {
        &mut self.inner
    }

    fn inner(&self) -> &OsString {
        &self.inner
    }
}

impl PathLike for AbsPath {
    // TODO
}

impl From<&OsStr> for AbsPath {
    fn from(value: &OsStr) -> Self {
        AbsPath {
            inner: super::sanitize_os_string(value, b"/"),
        }
    }
}
