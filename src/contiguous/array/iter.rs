use std::iter::{FusedIterator, TrustedLen};
use std::marker::PhantomData;
use std::mem;
use std::ptr::{self, NonNull};

use super::Array;
#[allow(unused)]
use crate::contiguous::Vector;

impl<T> IntoIterator for Array<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let result = IntoIter {
            ptr: self.ptr,
            left: self.size,
            _phantom: PhantomData
        };
        mem::forget(self);
        result
    }
}

/// An owned type for owned iteration over an [`Array`] or [`Vector`]. See [`Array::into_iter`] and
/// [`Vector::into_iter`].
pub struct IntoIter<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) left: usize,
    pub(crate) _phantom: PhantomData<T>
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for i in 0..self.left {
            unsafe { ptr::drop_in_place(self.ptr.add(i).as_ptr()) }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left > 0 {
            let value = unsafe { self.ptr.read() };
            self.ptr = unsafe { self.ptr.add(1) };
            self.left -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.left > 0 {
            let value = unsafe { self.ptr.add(self.left - 1).read() };
            self.left -= 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.left
    }
}

unsafe impl<T> TrustedLen for IntoIter<T> {}

// Just use the iter and iter_mut definitions provided by Deref<Target=[T]>.