//! Types for interacting with the directories of a file system. Simpler than the file module, this
//! one is primarily focussed on the [`Directory`] and [`DirEntry`] types.
// TODO: Write more docs.

mod dir;
mod dir_entry;

pub use dir::*;
pub use dir_entry::*;
