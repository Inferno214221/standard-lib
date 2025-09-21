//! Types for interacting with the directories of a file system. Simpler than the file module, this
//! one is primarily focussed on the [`Directory`] and [`DirEntry`] types.
//! 
//! This module provides the [`Directory`] type and associated types, including errors and
//! iterators.
//! 
//! # Opening
//! With less options relevant while opening `Directory`s, this module current does not provide a
//! builder like file's [`OpenOptions`](crate::fs::file::OpenOptions). This may change in the
//! future, if more relevant options are found.
//! 
//! # Access Mode
//! Unlike [`File`](crate::fs::File)s, `Directory`s are currently not associated with an access mode
//! to restrict operations at compile time. This may be changed in the future however, if it helps
//! to better represent the underlying entity and its functionality.
// TODO: Write more docs.

mod dir;
mod dir_entry;

pub use dir::*;
pub use dir_entry::*;
