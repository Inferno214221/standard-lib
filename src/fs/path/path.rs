use derive_more::{From, IsVariant, TryInto};

use crate::fs::path::{AbsPath, OwnedAbsPath, OwnedRelPath, PathLike};

#[derive(From, TryInto, IsVariant)]
pub enum OwnedPath {
    Absolute(OwnedAbsPath),
    Relative(OwnedRelPath),
}

use OwnedPath::*;

impl OwnedPath {
    pub fn root() -> OwnedAbsPath {
        OwnedAbsPath::root()
    }

    pub fn home() -> Option<OwnedAbsPath> {
        OwnedAbsPath::home()
    }

    pub fn cwd() -> Option<OwnedAbsPath> {
        OwnedAbsPath::cwd()
    }

    pub fn to_absolute<P: AsRef<AbsPath>>(self, base: P) -> OwnedAbsPath {
        match self {
            Absolute(abs) => abs,
            Relative(rel) => base.as_ref().join(&rel),
        }
    }
}

// pub enum Path {
//     Absolute(AbsPath),
//     Relative(RelPath),
// }

// use Path::*;
