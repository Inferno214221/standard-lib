//! Types for interacting with files (or file-like inodes) on a file system. Primarily revolves
//! around the [`File`] type and accompanying access mode markers.
//! 
//! This module provides the [`File`] type and various associated types, including markers, errors
//! and builders.
//!
//! # Opening
//! Although some convenience methods are provided on `File` itself, the full open functionality is
//! available via the [`OpenOptions`] builder (obtainable via [`File::options`]).
//! 
//! Files can be opened from an absolute path ([`Path<Abs>`](crate::fs::Path)), relative path and
//! directory ([`Path<Rel>`](crate::fs::Path), [`Directory`](crate::fs::Directory)) or a directory
//! entry ([`DirEntry`](crate::fs::dir::DirEntry)).
//!
//! # Closing
//! `File`s, despite not visibly implementing [`Drop`], ensure that the associated file is closed
//! when they are dropped, and can panic if an error occurs. To close a file with error handling,
//! the [`close`](File::close) method can be used instead.
//! 
//! It is also important to note that due to Linux file system behavior, closing a file does not
//! guarantee that it's data is written to disk. If this is important, please ensure that
//! [`sync`](File::sync) is called before closing.
//!
//! # Access Mode
//! Each file is associated with an AccessMode, which takes the form of a generic type parameter.
//! There are three available modes: [`ReadOnly`](super::ReadOnly), [`WriteOnly`](super::WriteOnly)
//! and [`ReadWrite`], representing those exposed by the Linux syscalls. Wherever applicable, the
//! default mode is `ReadWrite`. Along with these types, there are also two traits seen in the
//! public API, `Read` and `Write`. It is unlikely that these type need to be used directly, but
//! instead act as markers for the available modes to allow overlapping implications.

mod access;
mod error;
mod file;
mod options;

pub use access::*;
pub use error::*;
pub use file::*;
pub use options::*;
