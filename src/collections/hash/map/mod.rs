//! A module containing [`HashMap`] and associtated types.
//!
//! Currently, the only other included types are for iteration, providing owned and borrowed
//! iteration over entries, keys or values in a map.
//!
//! As a note, there is no mutable iterator over entries or keys because mutating the keys of a
//! HashMap in place would cause a logic error.
//!
//! [`HashMap`] is also re-exported under the parent module.

mod error;
mod hash_map;
mod iter;

pub use error::*;
pub use hash_map::*;
pub use iter::*;
