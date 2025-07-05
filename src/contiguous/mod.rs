//! Contiguous collection types. Namely [`Array`] and [`Vector`] for contiguous collections that
//! vary in size at runtime.

pub mod array;
pub mod vector;

#[doc(inline)]
pub use array::Array;
#[doc(inline)]
pub use vector::Vector;