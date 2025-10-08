//! Various general-purpose collection types.
//!
//! # Purpose
//! I wrote these types to learn about each of the data structures themselves, but also concepts
//! such as pointers, allocations, iterators and hashing.
//!
//! # Method
//! Applicable types here implement [`Deref<Target = [T]>`](std::ops::Deref) (and DerefMut), which
//! saves me from writing some of the more repetitive functionality.

// pub mod binary_tree;
pub mod contiguous;
pub mod hash;
pub mod linked;
pub mod traits;
