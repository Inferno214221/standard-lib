//! Contiguous collection types. Namely [`Array`] and [`Vector`] for contiguous collections that
//! vary in size at runtime.
#![warn(missing_docs)]

pub mod array;
pub mod vector;

#[doc(inline)]
pub use array::Array;
#[doc(inline)]
pub use vector::Vector;