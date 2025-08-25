use std::fmt::{self, Display, Formatter};

use crate::fs::path::{AbsPath, OwnedAbsPath, PathLike};

pub struct DisplayAbs<'a> {
    pub(crate) inner: &'a AbsPath,
}

pub struct DisplayFull<'a> {
    pub(crate) inner: &'a AbsPath,
}

pub struct DisplayHome<'a> {
    pub(crate) inner: &'a AbsPath,
}

impl<'a> DisplayAbs<'a> {
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

impl<'a> Display for DisplayAbs<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full())
    }
}

impl<'a> Display for DisplayFull<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.as_ref().to_string_lossy())
    }
}

impl<'a> Display for DisplayHome<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(home) = OwnedAbsPath::home()
            && let Some(rel) = self.inner.relative(&home) {
            write!(f, "~{}", rel.display().slash())
        } else {
            write!(f, "{}", self.inner.as_ref().to_string_lossy())
        }
    }
}
