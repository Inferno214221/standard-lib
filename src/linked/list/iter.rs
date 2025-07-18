use std::iter::{FusedIterator, TrustedLen};
use std::marker::PhantomData;

use ListState::*;

use super::{Link, LinkedList, ListContents, ListState};

// TODO: impl DoubleEndedIterator for all of these

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            curr: match self.state {
                Empty => None,
                Full(ListContents { head, .. }) => Some(head),
            },
            len: self.len(),
            _phantom: PhantomData,
        }
    }
}

pub struct IntoIter<T> {
    pub(crate) curr: Link<T>,
    pub(crate) len: usize,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        while let Some(ptr) = self.curr {
            self.curr = *ptr.next();
            // SAFETY: We only iterate forwards during this drop implementation, so all duplicate
            // NodeRefs are ignored and dropped.
            unsafe { ptr.drop_node(); }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            // Use a box to move the value and clean up.
            let node = ptr.take_node();
            self.curr = node.next;
            self.len -= 1;
            node.value
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

// SAFETY: IntoIter::size_hint returns the exact length of the iterator.
unsafe impl<T> TrustedLen for IntoIter<T> {}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            curr: match self.state {
                Empty => None,
                Full(ListContents { head, .. }) => Some(head),
            },
            len: self.len(),
            _phantom: PhantomData,
        }
    }
}

pub struct IterMut<'a, T> {
    pub(crate) curr: Link<T>,
    pub(crate) len: usize,
    pub(crate) _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            self.curr = *ptr.next();
            self.len -= 1;
            unsafe { &mut ptr.as_non_null().as_mut().value }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

// SAFETY: IterMut::size_hint returns the exact length of the iterator.
unsafe impl<'a, T> TrustedLen for IterMut<'a, T> {}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            curr: match self.state {
                Empty => None,
                Full(ListContents { head, .. }) => Some(head),
            },
            len: self.len(),
            _phantom: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    pub(crate) curr: Link<T>,
    pub(crate) len: usize,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            self.curr = *ptr.next();
            self.len -= 1;
            unsafe { &ptr.as_non_null().as_ref().value }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

// SAFETY: IterMut::size_hint returns the exact length of the iterator.
unsafe impl<'a, T> TrustedLen for Iter<'a, T> {}
