//! A module containing [`HashSet`] and associtated types.
//!
//! Some of these types provide owned and borrowed iteration over a set's elements while others are
//! iterators over the result of set operations on two HashSets.
//!
//! As a note, there is no mutable iterator over the elements of a set because mutating the entries
//! in place would cause a logic error.
//!
//! [`HashSet`] is also re-exported under the parent module.

mod hash_set;
mod iter;

pub use hash_set::*;
pub use iter::*;

#[doc(inline)]
pub use super::map::IndexNoCap;
