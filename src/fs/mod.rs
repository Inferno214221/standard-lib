//! File-system interface which acts as an abstraction over Linux's syscalls. As such, this crate is
//! specific to Linux only.
//!
//! # Purpose
//! Doing the research to write this module has taught me heaps about how Linux actually works and
//! how programs interact with it. It has also helped to clarify what functions the Kernel actually
//! encompasses.
//!
//! # Method
//! All components of this module are written to target Linux specifically. Although most parts
//! probably rely on the general POSIX standards, there may be some places where they make use of
//! Linux-specific extensions or implementation details. As for architecture, I haven't yet decided
//! whether I'm targeting 64-bit systems specifically or handling all pointer sizes, 64-bit is
//! certainly the primary target. Beyond pointer width, this should work on architecture thanks to
//! glibc's platform specific constants.
//!
//! Some types in the module very closely resemble [`std::fs`], but others differ significantly.
//! Notable differences include:
//! - A [`Directory`](dir::Directory) type, which leverages the file descriptor and open syscall's
//!   ability to open and refer to a directory by a descriptor rather then just a path. This could
//!   help to prevent TOCTOU bugs / exploits.
//! - Distinct absolute ([`Path<Abs>`](path::Path<Abs>)) and relative
//!   ([`Path<Rel>`](path::Path<Rel>)) path types with additional formatting invariants for both.
//!   Among other things, this prevents unexpected behavior such as absolute paths replacing each
//!   other when joined (as they do in [`std`]).
//! - Statically dispatched error types for more explicit error handling.
#![cfg(target_os = "linux")]
#![doc(cfg(Linux))]

pub mod dir;
pub mod error;
pub mod file;
pub(crate) mod panic;
pub mod path;
pub(crate) mod util;

mod fd;
mod file_type;
mod metadata;

pub(crate) use fd::*;
pub use file_type::*;
pub use metadata::*;

#[doc(inline)]
pub use dir::Directory;
#[doc(inline)]
pub use file::File;
#[doc(inline)]
pub use path::{Abs, OwnedPath, Path, Rel};