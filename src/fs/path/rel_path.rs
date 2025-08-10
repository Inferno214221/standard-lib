use std::ffi::OsString;

pub struct RelPath {
    pub(crate) inner: OsString,
}

impl From<String> for RelPath {
    fn from(value: String) -> Self {
        RelPath {
            inner: OsString::from(value),
        }
    }
}