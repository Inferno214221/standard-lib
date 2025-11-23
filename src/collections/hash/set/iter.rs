use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;

use super::HashSet;
use crate::collections::hash::map::{IntoKeys, Keys};
#[cfg(doc)]
use crate::collections::traits::set::SetIterator;

impl<T: Hash + Eq, B: BuildHasher> IntoIterator for HashSet<T, B> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.inner.into_keys())
    }
}

/// A type for owned iteration over a [`HashSet`]. Produces values of type `T`.
///
/// See [`HashSet::into_iter`].
pub struct IntoIter<T: Hash + Eq>(pub(crate) IntoKeys<T, ()>);

impl<T: Hash + Eq> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: Hash + Eq> FusedIterator for IntoIter<T> {}

impl<'a, T: Hash + Eq, B: BuildHasher> IntoIterator for &'a HashSet<T, B> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.keys())
    }
}

/// A type for borrowed iteration over a [`HashSet`]. Produces values of type `&T`.
///
/// See [`HashSet::iter`].
pub struct Iter<'a, T: Hash + Eq>(pub(crate) Keys<'a, T, ()>);

impl<'a, T: Hash + Eq> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: Hash + Eq> FusedIterator for Iter<'a, T> {}
