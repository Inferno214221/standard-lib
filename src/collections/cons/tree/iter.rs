use std::{mem, rc::Rc};

use super::{ConsTree, ConsTreeNode};

#[derive(Clone)]
pub struct Iter<'a, T> {
    pub(crate) inner: Option<&'a ConsTreeNode<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let ConsTreeNode { value, next } = self.inner?;
        self.inner = next.inner.as_deref();
        Some(value)
    }
}

pub struct OwnedIter<T: Clone> {
    pub(crate) inner: ConsTree<T>,
}

impl<T: Clone> OwnedIter<T> {
    /// Returns all remaining elements of this iterator, as a `ConsTree`.
    pub fn remainder(self) -> ConsTree<T> {
        self.inner
    }
}

impl<T: Clone> Iterator for OwnedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_to_owned()
    }
}

pub struct UniqueIter<T> {
    pub(crate) inner: ConsTree<T>,
}

impl<T> UniqueIter<T> {
    /// Returns all remaining elements of this iterator, as a [`ConsTree`]. When used on an
    /// exhausted `UniqueIter`, the list returned will contain all the shared items (of which there
    /// may be none).
    pub fn remainder(self) -> ConsTree<T> {
        self.inner
    }
}

impl<T> Iterator for UniqueIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_if_unique()
    }
}

pub struct RcIter<T> {
    pub(crate) inner: ConsTree<T>,
}

impl<T> RcIter<T> {
    /// Returns all remaining elements of this iterator, as a `ConsTree`.
    pub fn remainder(self) -> ConsTree<T> {
        self.inner
    }
}

impl<T> Iterator for RcIter<T> {
    type Item = Rc<ConsTreeNode<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = mem::take(&mut self.inner.inner);

        match inner {
            Some(rc) => {
                self.inner.inner = rc.next.inner.clone();
                Some(rc)
            },
            None => {
                self.inner.inner = inner;
                None
            },
        }
    }
}