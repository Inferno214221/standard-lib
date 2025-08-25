use std::{env, ffi::{OsStr, OsString}};

use crate::fs::path::{self, sealed::PathInternals, PathLike};

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

// TODO: There are a lot of invariants that make taking a slice difficult to do.

/// TODO
///
/// # Invariants
/// - The string starts with '/'.
/// - The string contains no repeated '/' characters or occurrences of "/./".
/// - The string contains no trailing '/'.
/// - The string contains no \0.
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

impl PathInternals for OwnedAbsPath {
    fn inner_mut(&mut self) -> &mut OsString {
        &mut self.inner
    }

    fn inner(&self) -> &OsString {
        &self.inner
    }
}

impl PathLike for OwnedAbsPath {
    // TODO
}

impl From<&OsStr> for OwnedAbsPath {
    fn from(value: &OsStr) -> Self {
        OwnedAbsPath {
            inner: path::sanitize_os_string(value, b"/"),
        }
    }
}
