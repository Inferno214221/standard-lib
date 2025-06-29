use std::marker::PhantomData;

use super::{DoublyLinkedList, ListState, Link, Inner};

use ListState::*;

// FIXME: implement drop for all of these types

impl<T> IntoIterator for DoublyLinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            curr: match self.state {
                Empty => None,
                Full(Inner { head, .. }) => Some(head),
            },
            index: 0,
            len: self.len(),
            _phantom: PhantomData
        }
    }
}

pub struct IntoIter<T> {
    curr: Link<T>,
    index: usize,
    len: usize,
    _phantom: PhantomData<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            // Use a box to move the value and clean up.
            let node = ptr.take_node();
            self.curr = node.next;
            self.index += 1;
            node.value
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.len))
    }
}

// TODO: impl DoubleEndedIterator, FusedIterator, TrustedLen, Drop

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoublyLinkedList<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut DoublyLinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            curr: match self.state {
                Empty => None,
                Full(Inner { head, .. }) => Some(head),
            },
            index: 0,
            len: self.len(),
            _phantom: PhantomData
        }
    }
}

pub struct IterMut<'a, T> {
    curr: Link<T>,
    index: usize,
    len: usize,
    _phantom: PhantomData<&'a mut T>
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            self.curr = *ptr.next();
            self.index += 1;
            unsafe { &mut ptr.as_non_null().as_mut().value }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.len))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<'a, T> IntoIterator for &'a DoublyLinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            curr: match self.state {
                Empty => None,
                Full(Inner { head, .. }) => Some(head),
            },
            index: 0,
            len: self.len(),
            _phantom: PhantomData
        }
    }
}

pub struct Iter<'a, T> {
    curr: Link<T>,
    index: usize,
    len: usize,
    _phantom: PhantomData<&'a T>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr.map(|ptr| {
            self.curr = *ptr.next();
            self.index += 1;
            unsafe { &ptr.as_non_null().as_ref().value }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.len))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}