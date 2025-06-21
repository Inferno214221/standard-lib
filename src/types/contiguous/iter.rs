use std::marker::PhantomData;

use super::{Array, Vector};

pub struct IntoIter<T> {
    arr: Array<T>,
    index: usize,
    _phantom: PhantomData<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arr.size {
            let value = unsafe { self.arr.ptr.add(self.index).read() };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.arr.size))
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            arr: self,
            index: 0,
            _phantom: PhantomData
        }
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Array::from(self).into_iter()
    }
}

// Just use the iter and iter_mut definitions provided by Deref<Target=[T]>.