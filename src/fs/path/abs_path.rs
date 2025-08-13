use std::ffi::{OsStr, OsString};

use crate::fs::path::{sealed::PathInternals, PathLike};

/// TODO
/// 
/// # Invariants
/// - The string starts with '/'.
/// - The string contains no repeated '/' characters or occurances of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0 but is followed by exactly one.
pub struct AbsPath {
    pub(crate) inner: OsString,
}

impl AbsPath {
    // TODO: pub fn root() -> AbsPath;

    // TODO: pub fn home() -> Option<AbsPath>;

    // TODO: pub fn cwd() -> Option<AbsPath>;
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