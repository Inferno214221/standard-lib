use std::ffi::OsStr;

use crate::fs::path::{AbsPath, RelPath};

pub enum Path {
    Absolute(AbsPath),
    Relative(RelPath),
}

use Path::*;

// impl Path {
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
// }