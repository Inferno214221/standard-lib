use std::{ffi::OsStr, num::NonZero, str::FromStr};

use crate::fs::error::{EmptyStrError, HomeResolutionError};

use super::{Abs, OwnedPath, Path, PathParseError, Rel};

/// An OwnedPath with a statically dispatched state. TODO
// TODO: More docs here.
pub enum DispatchedPath {
    Abs(OwnedPath<Abs>),
    Rel(OwnedPath<Rel>),
}

impl FromStr for DispatchedPath {
    // TODO: Is it fair to sanitize in a parse implementation?
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('/') => {
                Ok(DispatchedPath::Abs(
                    OwnedPath::from(s)
                ))
            },
            // TODO: Handle '~' more consistently.
            Some('~') => {
                Ok(DispatchedPath::Abs(
                    OwnedPath::from(&s[1..])
                        .resolve_home()
                        .ok_or(HomeResolutionError)?
                ))
            },
            // '.' is caught here, so "./" matches and is then sanitized to a relative "/".
            Some(_) => {
                Ok(DispatchedPath::Rel(
                    OwnedPath::from(s)
                ))
            },
            None => Err(EmptyStrError)?,
        }
    }
}

macro_rules! dispatch_ref {
    ($this:ident.$name:ident) => {
        match $this {
            DispatchedPath::Abs(abs) => abs.$name(),
            DispatchedPath::Rel(rel) => rel.$name(),
        }
    };
}

impl DispatchedPath {
    pub fn abs_or_resolve(self, target: OwnedPath<Abs>) -> OwnedPath<Abs> {
        match self {
            DispatchedPath::Abs(abs) => abs,
            DispatchedPath::Rel(rel) => rel.resolve(target),
        }
    }

    pub fn rel_or_make_relative<P: AsRef<Path<Abs>>>(self, target: P) -> OwnedPath<Rel> {
        match self {
            DispatchedPath::Abs(abs) => abs.make_relative(target),
            DispatchedPath::Rel(rel) => rel,
        }
    }

    pub fn len(&self) -> NonZero<usize> {
        dispatch_ref!(self.len)
    }

    pub fn as_os_str(&self) -> &OsStr {
        dispatch_ref!(self.as_os_str)
    }

    pub fn as_os_str_no_lead(&self) -> &OsStr {
        dispatch_ref!(self.as_os_str_no_lead)
    }

    pub fn as_bytes(&self) -> &[u8] {
        dispatch_ref!(self.as_bytes)
    }

    pub fn basename(&self) -> &OsStr {
        dispatch_ref!(self.basename)
    }
}

impl From<OwnedPath<Abs>> for DispatchedPath {
    fn from(value: OwnedPath<Abs>) -> Self {
        DispatchedPath::Abs(value)
    }
}

impl From<OwnedPath<Rel>> for DispatchedPath {
    fn from(value: OwnedPath<Rel>) -> Self {
        DispatchedPath::Rel(value)
    }
}

// TODO: Forwarding to generic OwnedPath and Path methods.