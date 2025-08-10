use std::ffi::OsString;

pub struct AbsPath {
    pub(crate) inner: OsString,
}

impl From<String> for AbsPath {
    fn from(value: String) -> Self {
        AbsPath {
            inner: OsString::from(value),
        }
    }
}