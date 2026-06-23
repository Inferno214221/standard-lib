use std::mem;

use crate::rc::brc::{Backend, Brc};

use super::{ConsBranch, ConsNode};

impl<'a, T, B: Backend> IntoIterator for &'a ConsBranch<T, B> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T, B>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.inner.as_deref(),
        }
    }
}

/// See [`ConsBranch::iter`].
pub struct Iter<'a, T, B: Backend> {
    pub(crate) inner: Option<&'a ConsNode<T, B>>,
}

impl<'a, T, B: Backend> Iterator for Iter<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let ConsNode { value, next } = self.inner?;
        self.inner = next.inner.as_deref();
        Some(value)
    }
}

impl<'a, T, B: Backend> Clone for Iter<'a, T, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
        }
    }
}

/// See [`ConsBranch::into_iter_owned`].
pub struct OwnedIter<T: Clone, B: Backend> {
    pub(crate) inner: ConsBranch<T, B>,
}

impl<T: Clone, B: Backend> OwnedIter<T, B> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`].
    pub fn remainder(self) -> ConsBranch<T, B> {
        self.inner
    }
}

impl<T: Clone, B: Backend> Iterator for OwnedIter<T, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_to_owned()
    }
}

/// See [`ConsBranch::into_iter_unique`].
pub struct UniqueIter<T, B: Backend> {
    pub(crate) inner: ConsBranch<T, B>,
}

impl<T, B: Backend> UniqueIter<T, B> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`]. When used on an
    /// exhausted `UniqueIter`, the list returned will contain all the shared items (of which there
    /// may be none).
    pub fn remainder(self) -> ConsBranch<T, B> {
        self.inner
    }
}

impl<T, B: Backend> Iterator for UniqueIter<T, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_if_unique()
    }
}

/// See [`ConsBranch::into_iter_rc`].
pub struct BrcIter<T, B: Backend> {
    pub(crate) inner: ConsBranch<T, B>,
}

impl<T, B: Backend> BrcIter<T, B> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`].
    pub fn remainder(self) -> ConsBranch<T, B> {
        self.inner
    }
}

impl<T, B: Backend> Iterator for BrcIter<T, B> {
    type Item = Brc<ConsNode<T, B>, B>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = mem::take(&mut self.inner.inner);

        match inner {
            Some(rc) => {
                self.inner = rc.next.clone();
                Some(rc)
            },
            None => {
                self.inner.inner = inner;
                None
            },
        }
    }
}