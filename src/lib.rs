#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]
#![feature(trusted_len)]
#![feature(debug_closure_helpers)]
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
