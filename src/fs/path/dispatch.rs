use std::str::FromStr;

use crate::fs::error::{EmptyStrError, HomeResolutionError};

use super::{Abs, OwnedPath, Path, PathParseError, Rel};

/// An OwnedPath with a statically dispatched state. TODO
// TODO: More docs here.
pub enum DispatchedPath {
    // No niche optimization for some reason? I thought OsString would have a null niche.
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
}

// TODO: Forwarding to generic OwnedPath and Path methods.