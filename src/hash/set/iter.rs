use std::{hash::{BuildHasher, Hash}, iter::Chain};

use super::HashSet;

use crate::hash::map::{IntoKeys, Keys};

impl<T: Hash + Eq, B: BuildHasher> IntoIterator for HashSet<T, B> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.inner.into_keys())
    }
}

pub struct IntoIter<T: Hash + Eq> (
    pub(crate) IntoKeys<T, ()>,
);

impl<T: Hash + Eq> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T: Hash + Eq, B: BuildHasher> IntoIterator for &'a HashSet<T, B> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.keys())
    }
}

pub struct Iter<'a, T: Hash + Eq> (
    pub(crate) Keys<'a, T, ()>,
);

impl<'a, T: Hash + Eq> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct Difference<'a, T: Hash + Eq, B: BuildHasher> {
    pub(crate) inner: Iter<'a, T>,
    pub(crate) other: &'a HashSet<T, B>
}

impl<'a, T: Hash + Eq, B: BuildHasher> Iterator for Difference<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = next && self.other.contains(item) {
            next = self.inner.next();
        }
        next
    }
}

pub struct SymmetricDifference<'a, T: Hash + Eq, B: BuildHasher> {
    pub(crate) inner: Chain<Difference<'a, T, B>, Difference<'a, T, B>>
}

impl<'a, T: Hash + Eq, B: BuildHasher> Iterator for SymmetricDifference<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct Intersection<'a, T: Hash + Eq, B: BuildHasher> {
    pub(crate) inner: Iter<'a, T>,
    pub(crate) other: &'a HashSet<T, B>
}

impl<'a, T: Hash + Eq, B: BuildHasher> Iterator for Intersection<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = next && !self.other.contains(item) {
            next = self.inner.next();
        }
        next
    }
}

pub struct Union<'a, T: Hash + Eq, B: BuildHasher> {
    pub(crate) inner: Chain<Iter<'a, T>, Difference<'a, T, B>>
}

impl<'a, T: Hash + Eq, B: BuildHasher> Iterator for Union<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}