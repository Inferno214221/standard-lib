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

    pub fn to_absolute(self, base: &AbsPath) -> OwnedAbsPath {
        match self {
            Absolute(abs) => abs,
            Relative(rel) => base.join(&rel),
        }
    }
}
    
//     delegate! { to match self {
//         Absolute(path) => path,
//         Relative(path) => path,
//     } {
//         pub fn len(&self) -> usize;

//         pub fn is_empty(&self) -> bool;

//         pub fn as_os_str(&self) -> &OsStr;

//         pub fn as_bytes(&self) -> &[u8];

//         pub fn as_bytes_with_null(&self) -> &[u8];

//         pub fn as_ptr(&self) -> *const u8;

//         pub fn join(&mut self, other: &RelPath);
//     }}

// pub enum Path {
//     Absolute(AbsPath),
//     Relative(RelPath),
// }

// use Path::*;
