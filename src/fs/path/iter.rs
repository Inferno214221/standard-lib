use std::ffi::OsStr;

use super::PathLike;

pub struct Components<'a> {
    bytes: &'a [u8],
}

// impl<'a, P: PathLike> IntoIterator for &'a P {
//     type Item = &'a OsStr;

//     type IntoIter = Components<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         Components {
//             bytes: self.as_bytes(),
//         }
//     }
// }

// impl<'a> Iterator for Components<'a> {
//     type Item = &'a OsStr;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!("Create slice over the next sequence of non-'/' characters")
//     }
// }