use std::{mem, rc::Rc};

use super::{ConsBranch, ConsNode};

/// See [`ConsBranch::iter`].
pub struct Iter<'a, T> {
    pub(crate) inner: Option<&'a ConsNode<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let ConsNode { value, next } = self.inner?;
        self.inner = next.inner.as_deref();
        Some(value)
    }
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// See [`ConsBranch::into_iter_owned`].
pub struct OwnedIter<T: Clone> {
    pub(crate) inner: ConsBranch<T>,
}

impl<T: Clone> OwnedIter<T> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`].
    pub fn remainder(self) -> ConsBranch<T> {
        self.inner
    }
}

impl<T: Clone> Iterator for OwnedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_to_owned()
    }
}

/// See [`ConsBranch::into_iter_unique`].
pub struct UniqueIter<T> {
    pub(crate) inner: ConsBranch<T>,
}

impl<T> UniqueIter<T> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`]. When used on an
    /// exhausted `UniqueIter`, the list returned will contain all the shared items (of which there
    /// may be none).
    pub fn remainder(self) -> ConsBranch<T> {
        self.inner
    }
}

impl<T> Iterator for UniqueIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_if_unique()
    }
}

/// See [`ConsBranch::into_iter_rc`].
pub struct RcIter<T> {
    pub(crate) inner: ConsBranch<T>,
}

impl<T> RcIter<T> {
    /// Returns all remaining elements of this iterator, as a [`ConsBranch`].
    pub fn remainder(self) -> ConsBranch<T> {
        self.inner
    }
}

impl<T> Iterator for RcIter<T> {
    type Item = Rc<ConsNode<T>>;

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