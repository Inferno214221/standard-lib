use std::{mem::{self, MaybeUninit}, ptr::NonNull};

use crate::collections::circular::CircStack;

impl<T, const N: usize> IntoIterator for CircStack<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.forget_init(),
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct IntoIter<T, const N: usize> {
    pub(crate) inner: CircStack<MaybeUninit<T>, N>,
    // Index here refers to the next index to retrieve a value from, or N if exhausted.
    pub(crate) index: usize,
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        for i in self {
            drop(i);
        }
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // We use N as a guard value, because it will always be available but unreachable.
        if self.index >= N {
            return None
        }

        let value = unsafe {
            mem::replace(&mut self.inner[self.index], MaybeUninit::uninit()).assume_init()
        };

        self.index += 1;

        Some(value)
    }
}

#[derive(Debug)]
pub struct IterMut<'a, T, const N: usize> {
    // pub(crate) inner: &'a mut CircStack<T, N>,
    pub(crate) buffer: &'a mut [T; N],
    pub(crate) last: usize,
    // Index here refers to the next index to retrieve a value from, or N if exhausted.
    pub(crate) index: usize,
}

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // // We use N as a guard value, because it will always be available but unreachable.
        // if self.index >= N {
        //     return None
        // }

        // let value = self.buffer.index_mut(self.index);

        // self.index = super::increment::<N>(self.index);

        // if self.index == self.last {
        //     self.index = N;
        // }

        // Some(value)

        if self.index >= N {
            return None
        }

        let head: NonNull<T> = NonNull::from_mut(&mut self.buffer).cast();
        let mut ptr = unsafe { head.offset(self.index as isize) };

        // Iterate backwards, stopping if we hit the point where we started.
        self.index = super::sub_wrapping::<N>(self.index, 1);

        if self.index == self.last {
            self.index = N;
        }

        Some(unsafe { ptr.as_mut() })
    }
}

#[derive(Debug)]
pub struct Iter<'a, T, const N: usize> {
    pub(crate) inner: &'a CircStack<T, N>,
    // Index here refers to the next index to retrieve a value from, or N if exhausted.
    pub(crate) index: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // We use N as a guard value, because it will always be available but unreachable.
        if self.index >= N {
            return None
        }

        let value = &self.inner[self.index];

        self.index += 1;

        Some(value)
    }
}