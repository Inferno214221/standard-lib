use std::iter::{FusedIterator, TrustedLen};
use std::marker::PhantomData;
use std::mem;
use std::ptr::{self, NonNull};

use super::{Array, Vector};

impl<T> IntoIterator for Array<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let result = IntoIter {
            ptr: dbg!(self.ptr),
            left: dbg!(self.size),
            _phantom: PhantomData
        };
        mem::forget(self);
        result
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Array::from(self).into_iter()
    }
}

/// An owned type for owned iteration over an [`Array`] or [`Vector`]. See [`Array::into_iter`] and
/// [`Vector::into_iter`].
pub struct IntoIter<T> {
    ptr: NonNull<T>,
    left: usize,
    _phantom: PhantomData<T>
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