//! Types for representing valid file system paths. [`OwnedPath`] and [`Path`] are available as
//! owned and slice representations, respectively.
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
//! For most file operations, an [`Abs`] path will be required, meaning that relative paths need to
//! be [`resolved`](Path<Rel>::resolve) first.
//! 
//! # Validity
//! Both path types uphold the following invariants to ensure that the contained
//! [`OsString`](std::ffi::OsString) is _valid_:
//! - The string starts with `/`.
//! - The string contains no repeated `/` characters or occurrences of `/./`.
//! - The string contains no trailing `/`.
//! - The string contains no `\0`.
//! 
//! Although these invariants are relatively strict, constructing an `OwnedPath` from an `&OsStr` or
//! `&str` is infallible because it sanitizes any invalid string provided. On the other hand,
//! constructing a `Path` from another slice type can fail and may do so relatively often, because
//! it won't mutate the original value, only verify that it is already valid.
//! 
//! # Instantiation
//! | Method | Output | Input | Description |
//! |-|-|-|-|
//! | [`OwnedPath::from`] | `OwnedPath` | `AsRef<OsStr>` | Clones and sanitizes. |
//! | [`OwnedPath::from_unchecked`] | `OwnedPath` | `Into<OsString>` | Moves without sanitizing, **unsafe**. |
//! | [`Path::new`] | `Cow<_, Path>` | `AsRef<OsStr>` | Validates and sanitizes if required. |
//! | [`Path::from_checked`] | `Option<&Path>` | `AsRef<OsStr>` | Fallibly validates. |
//! | [`Path::from_unchecked`] | `&Path` | `AsRef<OsStr>` | Coerces without sanitizing, **unsafe**. |
//! | [`Path::from_unchecked_mut`] | `&mut Path` | `AsMut<OsStr>` | Coerces mutably without sanitizing, **unsafe**. |
//! 
//! # Ensuring Existence
//! A path being valid doesn't ensure that it exists. TODO
//! 
//! # Ownership
//! Most methods which use Paths will take arguments bound by one the following:
//! - `T: AsRef<Path<_>>` - for borrowed data, allowing anything which can be converted to a
//!   `&Path`. This is used by any path manipulation methods that don't require ownership.
//! - `T: Into<OwnedPath<_>>` - for owned data, allowing either an owned or borrowed `Path` to be
//!   provided and cloned when required. Many file system operation do this, be cause they need to
//!   extend the underlying OsString to make it valid for passing to glibc functions.
// TODO: determine approach for existence checking methods and document TOCTOU choices.
#[cfg(doc)]
use std::convert::From;

mod abs;
mod dispatch;
mod display;
mod error;
mod iter;
mod path;
mod rel;

pub use abs::*;
pub use dispatch::*;
pub use display::*;
pub use error::*;
pub use iter::*;
pub use path::*;
pub use rel::*;