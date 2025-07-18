//! A module containing [`Vector`] and associtated types.
//!
//! Currently, the only other included type is [`IntoIter`] for owned iteration over an Vector, which
//! is a re-export of [`array::IntoIter`](super::array::IntoIter). [`IterMut`](std::slice::IterMut)
//! and [`Iter`](std::slice::Iter) from [`std::slice`] are used for borrowed iteration.
//!
//! [`Vector`] is also re-exported under the parent module.

mod iter;
mod vector;

pub use iter::*;
pub use vector::*;
