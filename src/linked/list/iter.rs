use std::iter::{FusedIterator, TrustedLen};
use std::marker::PhantomData;

use ListState::*;

use super::{LinkedList, ListContents, ListState};

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            list: self,
        }
    }
}

pub struct IntoIter<T> {
    // There is no point me rewriting all of this when the iterator can just hold the list and call
    // pop front/back.
    pub(crate) list: LinkedList<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len()
    }
}

// SAFETY: IntoIter::size_hint returns the exact length of the iterator.
unsafe impl<T> TrustedLen for IntoIter<T> {}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            state: self.state.clone(),
            _phantom: PhantomData,
        }
    }
}

pub struct IterMut<'a, T> {
    // Although ths fields are exactly the same as a list, this structure doesn't modify the
    // underlying nodes and uses len to track the number of items left to yield.
    pub(crate) state: ListState<T>,
    pub(crate) _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, head, .. }) => {
                let value = head.value_mut();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the first element is
                        // preceded by at least one more.
                        let new_head = unsafe { head.next().unwrap_unchecked() };
                        *head = new_head;
                        // Never actually modify the node itself.
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(value)
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, tail, .. }) => {
                let value = tail.value_mut();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = unsafe { tail.prev().unwrap_unchecked() };
                        *tail = new_tail;
                        // Never actually modify the node itself.
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(value)
            },
        }
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

// SAFETY: IterMut::size_hint returns the exact length of the iterator.
unsafe impl<'a, T> TrustedLen for IterMut<'a, T> {}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            state: self.state.clone(),
            _phantom: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    pub(crate) state: ListState<T>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, head, .. }) => {
                let value = head.value();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the first element is
                        // preceded by at least one more.
                        let new_head = unsafe { head.next().unwrap_unchecked() };
                        *head = new_head;
                        // Never actually modify the node itself.
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(value)
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, tail, .. }) => {
                let value = tail.value();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = unsafe { tail.prev().unwrap_unchecked() };
                        *tail = new_tail;
                        // Never actually modify the node itself.
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(value)
            },
        }
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

// SAFETY: IterMut::size_hint returns the exact length of the iterator.
unsafe impl<'a, T> TrustedLen for Iter<'a, T> {}
