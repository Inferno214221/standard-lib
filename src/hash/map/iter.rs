use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;
use std::slice::Iter as ArrIter;
use std::slice::IterMut as ArrIterMut;

use super::{Bucket, HashMap};
use crate::contiguous::array::IntoIter as ArrIntoIter;

impl<K: Hash + Eq, V, B: BuildHasher> IntoIterator for HashMap<K, V, B> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len(),
            inner: self.arr.into_iter(),
        }
    }
}

/// A type for owned iteration over a [`HashMap`]. Produces values of type `(K, V)`.
///
/// See [`HashMap::into_iter`].
pub struct IntoIter<K, V> {
    pub(crate) inner: ArrIntoIter<Bucket<K, V>>,
    pub(crate) len: usize,
}

impl<K: Hash + Eq, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        next.flatten()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
    }
}

impl<K: Hash + Eq, V> FusedIterator for IntoIter<K, V> {}

impl<'a, K: Hash + Eq, V, B: BuildHasher> IntoIterator for &'a HashMap<K, V, B> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            len: self.len(),
            inner: self.arr.iter(),
        }
    }
}

/// A type for borrowed iteration over a [`HashMap`]. Produces values of type `(&K, &V)`.
///
/// See [`HashMap::iter`].
pub struct Iter<'a, K, V> {
    pub(crate) inner: ArrIter<'a, Bucket<K, V>>,
    pub(crate) len: usize,
}

impl<'a, K: Hash + Eq, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        // Convert &'a (K, V) to (&'a K, &'a V) to avoid promising the internal use of a tuple.
        next.and_then(|i| i.as_ref().map(|(k, v)| (k, v)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
    }
}

impl<'a, K: Hash + Eq, V> FusedIterator for Iter<'a, K, V> {}

/// A type for owned iteration over a [`HashMap`]'s keys. Produces values of type `K`.
///
/// See [`HashMap::into_keys`].
pub struct IntoKeys<K, V>(pub(crate) IntoIter<K, V>);

impl<K: Hash + Eq, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: Hash + Eq, V> FusedIterator for IntoKeys<K, V> {}

/// A type for borrowed iteration over a [`HashMap`]'s keys. Produces values of type `&K`.
///
/// See [`HashMap::keys`].
pub struct Keys<'a, K, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K: Hash + Eq, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K: Hash + Eq, V> FusedIterator for Keys<'a, K, V> {}

/// A type for owned iteration over a [`HashMap`]'s values. Produces values of type `V`.
///
/// See [`HashMap::into_values`].
pub struct IntoValues<K, V>(pub(crate) IntoIter<K, V>);

impl<K: Hash + Eq, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: Hash + Eq, V> FusedIterator for IntoValues<K, V> {}

/// A type for mutable iteration over a [`HashMap`]'s values. Produces values of type `&mut V`.
///
/// See [`HashMap::values_mut`].
pub struct ValuesMut<'a, K, V> {
    pub(crate) inner: ArrIterMut<'a, Bucket<K, V>>,
    pub(crate) len: usize,
}

impl<'a, K: Hash + Eq, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        next.and_then(|i| i.as_mut().map(|(_, v)| v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len))
    }
}

impl<'a, K: Hash + Eq, V> FusedIterator for ValuesMut<'a, K, V> {}

/// A type for borrowed iteration over a [`HashMap`]'s values. Produces values of type `&V`.
///
/// See [`HashMap::values`].
pub struct Values<'a, K, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K: Hash + Eq, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K: Hash + Eq, V> FusedIterator for Values<'a, K, V> {}
