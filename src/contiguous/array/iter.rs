use std::iter::{FusedIterator, TrustedLen};
use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::{alloc, mem};

use super::Array;
#[allow(unused)]
use crate::contiguous::Vector;

impl<T> IntoIterator for Array<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let head = self.ptr.as_ptr().cast_const();
        let result = IntoIter {
            ptr: self.ptr,
            size: self.size,
            head,
            // Don't subtract 1 for the tail.
            // SAFETY: Offset of one won't overflow isize::MAX. The memory range between head and
            // tail will be the same as that of the Array, where the initial value of tail is never
            // read.
            tail: unsafe { head.add(self.size) },
            _phantom: PhantomData,
        };
        mem::forget(self);
        result
    }
}

/// A type for owned iteration over an [`Array`] or [`Vector`]. Produces values of type `T`.
///
/// See [`Array::into_iter`] and [`Vector::into_iter`].
pub struct IntoIter<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) size: usize,
    pub(crate) head: *const T, // Head points to the first element.
    pub(crate) tail: *const T, // Tail points one after the last element.
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        while self.head < self.tail {
            // SAFETY: The pointer is nonnull, properly aligned and valid for both reads and writes.
            // This method takes a mutable reference to self, so the underlying data can't be
            // mutated while it executes.
            unsafe { ptr::drop_in_place(self.head.cast_mut()) }
            // SAFETY: Offset of one won't overflow isize::MAX. If the resulting pointer meets
            // tail it won't ever be read, preventing reads out of bounds of the original Array.
            self.head = unsafe { self.head.add(1) };
        }

        let layout = Array::<T>::make_layout(self.size);

        if layout.size() != 0 {
            // SAFETY: ptr is ensured to be valid by the Array used to create this Iterator.
            // Zero-sized layouts aren't allocated and are guarded against deallocation.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast(), layout)
            }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head < self.tail {
            // SAFETY: The pointer is always valid for reads. We will increment the pointer next so
            // that the value is effectively moved off of the heap.
            let value = unsafe { self.head.read() };
            // SAFETY: Offset of one won't overflow isize::MAX. If the resulting pointer meets
            // tail it won't ever be read, preventing reads out of bounds of the original Array.
            self.head = unsafe { self.head.add(1) };
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.head < self.tail {
            // Tail sits one after the end, so we subtract first then read. Results in the same
            // number of operations and an accurate length calculation with subtraction.
            // SAFETY: Offset of one won't overflow isize::MAX. If the resulting pointer meets
            // head it won't ever be read, preventing reads out of bounds of the original Array.
            self.tail = unsafe { self.tail.sub(1) };
            // SAFETY: The pointer is always valid for reads and we've just decremented it so that
            // the previous value is never read and effectively moved off of the heap.
            Some(unsafe { self.tail.read() })
        } else {
            None
        }
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        // SAFETY: Both pointers are derived from the original allocation and aligned to multiple of
        // size_of::<T>(). The memory range between them is contained within the initial allocation.
        // tail is at worst equal to head, it can't be less than.
        unsafe { self.tail.offset_from_unsigned(self.head) }
    }
}

// SAFETY: IntoIter::size_hint returns the exact length of the iterator.
unsafe impl<T> TrustedLen for IntoIter<T> {}

// Just use the iter and iter_mut definitions provided by Deref<Target=[T]>.
