use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::os::unix::ffi::OsStrExt;

use super::{Abs, OwnedPath, Path, Rel, sealed};

pub struct DisplayPath<'a, State: sealed::PathState> {
    pub(crate) _phantom: PhantomData<fn() -> State>,
    pub(crate) inner: &'a Path<State>,
}

pub struct DisplayFull<'a> {
    pub(crate) inner: &'a Path<Abs>,
}

pub struct DisplayHome<'a> {
    pub(crate) inner: &'a Path<Abs>,
}

pub struct DisplayDotSlash<'a> {
    pub(crate) inner: &'a Path<Rel>,
}

pub struct DisplaySlash<'a> {
    pub(crate) inner: &'a Path<Rel>,
}

pub struct DisplayNoLead<'a> {
    pub(crate) inner: &'a Path<Rel>,
}

impl<'a> DisplayPath<'a, Abs> {
    pub const fn full(&self) -> DisplayFull<'a> {
        DisplayFull {
            inner: self.inner,
        }
    }

    pub const fn shrink_home(&self) -> DisplayHome<'a> {
        DisplayHome {
            inner: self.inner,
        }
    }
}

impl<'a> Display for DisplayPath<'a, Abs> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full())
    }
}

impl<'a> Display for DisplayFull<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.as_os_str().to_string_lossy())
    }
}

impl<'a> Display for DisplayHome<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(home) = OwnedPath::home()
            && let Some(rel) = self.inner.relative(&home) {
            write!(f, "~{}", rel.display().slash())
        } else {
            write!(f, "{}", self.inner.as_os_str().to_string_lossy())
        }
    }
}

impl<'a> DisplayPath<'a, Rel> {
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

impl<'a> Display for DisplayPath<'a, Rel> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dot_slash())
    }
}

impl<'a> Display for DisplayDotSlash<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.inner.as_os_str().to_string_lossy())
    }
}

impl<'a> Display for DisplaySlash<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.as_os_str().to_string_lossy())
    }
}

impl<'a> Display for DisplayNoLead<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", OsStr::from_bytes(
            &self.inner.as_os_str().as_bytes()[1..]
        ).to_string_lossy())
    }
}