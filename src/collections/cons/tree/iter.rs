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

impl<T: Clone> Iterator for OwnedIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_to_owned()
    }
}

pub struct UniqueIter<T> {
    pub(crate) inner: ConsTree<T>,
}

impl<T> Iterator for UniqueIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = mem::take(&mut self.inner.inner);

        match inner {
            Some(rc) => {
                // If this returns here, self.inner.inner has been replaced with a None.
                let ConsTreeNode { value, next } = Rc::into_inner(rc)?;
                self.inner.inner = next.inner;
                Some(value)
            },
            None => {
                self.inner.inner = inner;
                None
            },
        }
    }
}

pub struct RcIter<T> {
    pub(crate) inner: ConsTree<T>,
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