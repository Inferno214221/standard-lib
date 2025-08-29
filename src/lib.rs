//! This crate is my attempt at writing (some key parts of) a standard library.
//!
//! Also: I wrote a theme (and extensions) for rustdoc, check it out under "Settings".
//!
//! # Purpose
//! This repo / crate is a project that I'm working on as a learning experience, with no expectation
//! for it to be used in production. Writing these data structures helps me to understand and
//! appreciate them properly as well as scratching my "I could write that" itch.
//!
//! # Method
//! All data structures here are written based on my existing understanding and problem solving. I'm
//! not following any guides or copying from the standard library but neither am I restricting my
//! self from looking things up or referring to existing crates, especially their APIs. This project
//! isn't intended to copy Rust's [`std`] but rather takes a lot of inspiration from it.
//!
//! Although, I'm not writing this for production use, I intend to write it to a level where it
//! could be. I've been relatively cautious about unsafe code and panics, although there are almost
//! certainly some mistakes.
//!
//! # Error Handling
//! I'm still pretty new to Rust, so I'm still trying to pin down when and how to use [`Result`]s
//! effectively. Specifically for a standard library, it is more ergonomic for functions to panic in
//! some cases, because users don't want to be forced to handle an error every time they invoke a
//! method. For example, imagine having to handle the possibility of a capacity overflow every time
//! you push into a Vector. (The maximum capacity of a Vector on a 64-bit system is `9,223,372`
//! terabytes - more than Linux's maximum RAM capabilities. *I swear I didn't find that out the hard
//! way by writing a unit test for maximum capacity.*)
//!
//! When this crate employs errors via [`Result`]s, it does so in a method that is strongly typed,
//! using enums for static dispatch rather than dynamic, with structs (often ZSTs) that implement
//! [`Error`](std::error::Error) (Which apparently isn't a given).
//!
//! # Dependencies
//! At the moment, this crate uses `std`, which doesn't make sense for a real standard library. If I
//! write an allocator in the future, I might able to make this `#![no_std]` but until then, I'm
//! stuck with it. I'm not going to go an use [`Vec`] to write
//! [`Vector`](collections::contiguous::Vector) or anything. In fact, this library doesn't use
//! [`Vec`] at all.
//!
//! The [`fs`] module of this crate relies on `libc` for its thin syscall wrappers, providing strong
//! typing and portability.
//!
//! This crate also depends on some derive macros because they're helpful and remove the need for
//! some very repetitive programming.
//!
//! # Potential Future Additions
//! - Linux syscall-based components:
//!   - Allocation
//!   - Threads / Processes
//!   - Time
//!   - Basic Networking
//! - Data structures:
//!   - Binary Tree Map/Set
//!   - BTree Map/Set
#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]
#![feature(trusted_len)]
#![feature(debug_closure_helpers)]
#![feature(raw_os_error_ty)]
#![feature(ptr_as_ref_unchecked)]
#![feature(doc_cfg)]

// #![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::module_inception)]

pub mod collections;
pub mod fs;

pub(crate) mod util;
