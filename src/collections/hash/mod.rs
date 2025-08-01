//! Collections based on the [`Hash`](std::hash::Hash) trait, including [`HashMap`] and [`HashSet`]
//! for storing unique values or key-value pairs.
#![warn(missing_docs)]

pub mod map;
pub mod set;

#[doc(inline)]
pub use map::HashMap;
#[doc(inline)]
pub use set::HashSet;
