//! A module containing [`Array`] and associtated types.
//! 
//! Currently, the only other included type is [`IntoIter`] for owned iteration over an Array.
//! [`IterMut`](std::slice::IterMut) and [`Iter`](std::slice::Iter) from [`std::slice`] are used for
//! borrowed iteration.
//! 
//! [`Array`] is also re-exported under the parent module.

mod array;
mod iter;
mod tests;

pub use array::*;
pub use iter::*;
