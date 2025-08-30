//! Types for representing valid file system paths.
//! 
//! This module provides types which represent valid (although not necessarily existent) paths for
//! an operating system / file system. (Currently only for Linux, like the rest of [`fs`](super)).
//! 
//! # Approach
//! Paths are represented by one of two types, using the type-state pattern:
//! - [`OwnedPath`]: An owned mutable string representing a valid path.
//! - [`Path`]: A slice representing a valid path. (Note: The slice itself is valid, and isn't just
//!   a slice of a valid [`OwnedPath`].)
//! 
//! Each of these types is accompanied by a state, representing whether the path is absolute
//! ([`Abs`]) or relative ([`Rel`]). Both of these states are represented as zero-variant enums, so
//! they can't be instantiated.
//! 
//! # Validity
//! Both path types uphold the following invariants to ensure that the contained
//! [`OsString`](std::ffi::OsString) is _valid_:
//! - The string starts with `/`.
//! - The string contains no repeated `/` characters or occurrences of `/./`.
//! - The string contains no trailing `/`.
//! - The string contains no `\0`.
//! 
//! Although these invariants are relatively strict, constructing an `OwnedPath` from an `OsStr` or
//! `str` is infallible because it sanitizes any invalid string provided. On the other hand,
//! constructing a `Path` from another slice type can fail and may do so relatively often, because
//! it won't mutate the original value, only verify that it is already valid.
//! 
//! # Ensuring Existence
//! A path being valid doesn't ensure that it exists. TODO
// TODO: determine approach for existence checking methods and document TOCTOU choices.

mod abs;
mod display;
mod iter;
mod path;
mod rel;

pub use abs::*;
pub use display::*;
pub use iter::*;
pub use path::*;
pub use rel::*;