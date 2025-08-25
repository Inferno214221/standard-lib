use std::{ffi::OsStr, fmt::{self, Display, Formatter}};

use crate::fs::path::RelPath;

pub struct DisplayRel<'a> {
    pub(crate) inner: &'a RelPath,
}

pub struct DisplayDotSlash<'a> {
    pub(crate) inner: &'a RelPath,
}

pub struct DisplaySlash<'a> {
    pub(crate) inner: &'a RelPath,
}

pub struct DisplayNoLead<'a> {
    pub(crate) inner: &'a RelPath,
}

impl<'a> DisplayRel<'a> {
    pub const fn dot_slash(&self) -> DisplayDotSlash<'a> {
        DisplayDotSlash {
            inner: self.inner,
        }
    }

    pub const fn slash(&self) -> DisplaySlash<'a> {
        DisplaySlash {
            inner: self.inner,
        }
    }
    
    pub const fn no_lead(&self) -> DisplayNoLead<'a> {
        DisplayNoLead {
            inner: self.inner,
        }
    }
}

impl<'a> Display for DisplayRel<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dot_slash())
    }
}

impl<'a> Display for DisplayDotSlash<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.inner.as_ref().to_string_lossy())
    }
}

impl<'a> Display for DisplaySlash<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.as_ref().to_string_lossy())
    }
}

impl<'a> Display for DisplayNoLead<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe {
            OsStr::from_encoded_bytes_unchecked(
                &self.inner.as_ref().as_encoded_bytes()[1..]
            ).to_string_lossy()
        })
    }
}