use std::alloc;
use std::cmp;
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use super::Array;

/// A variable size contiguous collection, based on [`Array<T>`].
pub struct Vector<T> {
    pub(crate) arr: Array<MaybeUninit<T>>,
    pub(crate) len: usize
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

impl<T> Vector<T> {
    /// Returns the length of the Vector.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::Vector;
    /// let vec = Vector::from([1_u8, 2, 3]);
    /// assert_eq!(vec.len(), 3);
    /// ```
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the current capacity of the Vector. Unlike [`Vec`], the capacity is guaranteed to be
    /// exactly the value provided to any of the various capacity manipulation functions.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::Vector;
    /// let vec: Vector<u8> = Vector::with_cap(5);
    /// assert_eq!(vec.cap(), 5);
    /// ```
    pub const fn cap(&self) -> usize {
        self.arr.size()
    }

    /// Creates a new Vector with length and capacity 0. Memory will be allocated when the capacity
    /// changes.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::Vector;
    /// let vec: Vector<u8> = Vector::new();
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.cap(), 0);
    /// ```
    pub fn new() -> Vector<T> {
        Vector {
            arr: Array::new(),
            len: 0
        }
    }

    /// Creates a new Vector with capacity exactly equal to the provided value, allowing values to
    /// be added without reallocation.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::Vector;
    /// let mut vec: Vector<u8> = Vector::with_cap(5);
    /// assert_eq!(vec.cap(), 5);
    /// vec.extend([1_u8, 2, 3, 4, 5]);
    /// assert_eq!(vec.cap(), 5);
    /// ```
    pub fn with_cap(cap: usize) -> Vector<T> {
        Vector {
            arr: Array::new_uninit(cap),
            len: 0
        }
    }

    pub(crate) fn realloc_with_cap(
        &mut self,
        ptr: NonNull<MaybeUninit<T>>,
        old_cap: usize,
        new_cap: usize
    ) {
        if old_cap == new_cap { return; }

        let layout = Array::<MaybeUninit<T>>::make_layout(old_cap);

        let new_ptr = NonNull::new(
            unsafe {
                if old_cap == 0 {
                    // If the vec previously had a capacity of zero, we need a new allocation.
                    alloc::alloc(layout) as *mut MaybeUninit<T>
                } else {
                    // Otherwise, use realloc to handle moving or in-place size changing.
                    alloc::realloc(
                        ptr.as_ptr() as *mut u8,
                        layout,
                        new_cap
                    ) as *mut MaybeUninit<T>
                }
            }
        ).unwrap_or_else(|| alloc::handle_alloc_error(layout));

        // Prevent a double free by forgetting the old value of self.arr.
        mem::forget(mem::replace(
            &mut self.arr,
            unsafe { Array::from_parts(new_ptr, new_cap) }
        ));
    }

    pub(crate) fn grow(&mut self) {
        // Because we can't take the value from arr (invalidating it in the event of a panic), we
        // just clone the pointer and capacity. (Essentially two usizes)
        let Array { ptr, size: old_cap, .. } = self.arr;
        
        // SAFETY: old_cap < isize::MAX, so old_cap * 2 can't overflow. Can still exceed isize::MAX.
        let new_cap = cmp::max(old_cap * 2, 1);

        self.realloc_with_cap(ptr, old_cap, new_cap);
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap() {
            self.grow()
        }
        self.arr[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let value = unsafe {
                mem::replace(
                    &mut self.arr[self.len - 1],
                    MaybeUninit::uninit()
                ).assume_init()
            };
            self.len -= 1;
            Some(value)
        }
    }

    pub(crate) fn check_index(&self, index: usize) {
        assert!(
            index <= self.len,
            "Index {} out of bounds for Vector with len {}", index, self.len
        );
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.check_index(index);
        if self.len == self.cap() {
            self.grow()
        }

        let mut prev = MaybeUninit::new(value);
        for i in index..=self.len {
            prev = mem::replace(&mut self.arr[i], prev);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.check_index(index);

        let mut next = MaybeUninit::uninit();
        for i in (index..self.len).rev() {
            next = mem::replace(&mut self.arr[i], next);
        }

        self.len -= 1;
        unsafe { next.assume_init() }
    }

    pub fn replace(&mut self, index: usize, new_value: T) -> T {
        self.check_index(index);

        unsafe {
            mem::replace(
                &mut self.arr[index],
                MaybeUninit::new(new_value)
            ).assume_init()
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    // new_cap >= len + extra
    pub fn reserve(&mut self, extra: usize) {
        let Array { ptr, size: old_cap, .. } = self.arr;

        let new_cap = self.len.strict_add(extra);
        if new_cap <= old_cap { return; }

        self.realloc_with_cap(ptr, old_cap, new_cap);
    }

    pub fn shrink_to_fit(&mut self) {
        let Array { ptr, size: old_cap, .. } = self.arr;

        self.realloc_with_cap(ptr, old_cap, self.len);
    }

    pub fn adjust_cap(&mut self, new_cap: usize) {
        let Array { ptr, size: old_cap, .. } = self.arr;

        if new_cap < self.cap() {
            // Drop the values that are about to be deallocated.
            for i in new_cap..self.cap() {
                drop(
                    // SAFETY: count > isize::MAX is already guarded against and all possible values are
                    // within the allocated range of the Array.
                    unsafe { ptr.add(i).read() }
                );
                self.len -= 1;
            }
        }

        self.realloc_with_cap(ptr, old_cap, new_cap);
    }
}

impl<T> Extend<T> for Vector<T> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        for item in iter.into_iter() {
            self.push(item);
        }
    }

    fn extend_one(&mut self, item: T) {
        self.push(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    unsafe fn extend_one_unchecked(&mut self, item: T) where Self: Sized {
        self.arr[self.len] = MaybeUninit::new(item);
        self.len += 1;
    }
}

impl<T, I> From<I> for Vector<T> where I: IntoIterator<Item = T>, I::IntoIter: ExactSizeIterator {
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let mut vec = Vector::with_cap(iter.len());

        for item in iter {
            // SAFETY: Vec has been created with the right capacity.
            unsafe { vec.extend_one_unchecked(item) };
        }

        vec
    }
}

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vector::new();

        for item in iter {
            vec.push(item);
        }

        vec
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        // Call drop on all initialized values.
        for i in 0..self.len {
            drop(unsafe {
                self.arr.ptr.add(i).read().assume_init()
            });
        }

        // Then replace the array with an empty one to drop normally.
        drop(mem::take(&mut self.arr))
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(
                // Reinterpret *mut MaybeUninit<T> as *mut T for all values < len.
                self.arr.ptr.as_ptr() as *mut T,
                self.len
            )
        }
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(
                // Reinterpret *mut MaybeUninit<T> as *mut T for all values < len.
                self.arr.ptr.as_ptr() as *mut T,
                self.len
            )
        }
    }
}

impl<T: Clone> Clone for Vector<T> {
    fn clone(&self) -> Self {
        let mut vec = Self::with_cap(self.cap());

        for value in self.iter() {
            vec.push(value.clone());
        }

        vec
    }
}

impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vector")
            .field("contents", &&**self)
            .field("len", &self.len)
            .field("cap", &self.cap())
            .finish()
    }
}

impl<T> From<Vector<T>> for Array<T> {
    fn from(mut value: Vector<T>) -> Self {
        // Dealloc all uninit values > len.
        value.shrink_to_fit();

        // SAFETY: A Vector contains len initialized values with the same layout and alignment as an
        // Array.
        let arr = unsafe { mem::transmute_copy(&value.arr) };
        mem::forget(value);
        arr
    }
}

impl<T: Debug + Clone> From<Array<T>> for Vector<T> {
    fn from(value: Array<T>) -> Self {
        let len = value.size();
        Vector {
            // SAFETY: Array<MaybeUninit<T>> has the same layout as Array<T>.
            arr: unsafe { mem::transmute(value) },
            len
        }
    }
}