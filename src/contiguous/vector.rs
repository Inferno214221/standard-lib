use std::alloc;
use std::cmp;
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr;
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
    /// # use rust_basic_types::contiguous::Vector;
    /// let vec = Vector::from(1_u8..=3);
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
    /// # use rust_basic_types::contiguous::Vector;
    /// let vec: Vector<u8> = Vector::with_cap(5);
    /// assert_eq!(vec.cap(), 5);
    /// ```
    pub const fn cap(&self) -> usize {
        self.arr.size()
    }

    /// Returns true if the Vector contains no elements.
    ///  
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
    /// let mut vec: Vector<u8> = Vector::new();
    /// assert!(vec.is_empty());
    /// vec.push(1);
    /// assert!(!vec.is_empty())
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a new Vector with length and capacity 0. Memory will be allocated when the capacity
    /// changes.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
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
    /// # use rust_basic_types::contiguous::Vector;
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

    /// Push the provided value onto the end of the Vector, increasing the capacity if required.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
    /// let mut vec = Vector::<u8>::new();
    /// for i in 0..=5 {
    ///     vec.push(i);
    /// }
    /// assert_eq!(&*vec, &[0, 1, 2, 3, 4, 5]);
    /// ```
    pub fn push(&mut self, value: T) {
        if self.len == self.cap() {
            self.grow();
        }
        // SAFETY: The capacity has just been adjusted to support the addition of the new item.
        unsafe { self.push_unchecked(value) }
    }

    /// Push the provided value onto the end of the Vector, assuming that there is enough capacity
    /// to do so.
    /// 
    /// # Safety
    /// It is up to the caller to ensure that the Vector has enough capacity to add the provided
    /// value, using methods like [`reserve`](Vector::reserve), [`adjust_cap`](Vector::adjust_cap)
    /// or [`with_cap`](Vector::with_cap) to do so. Using this method on a Vector without enough
    /// capacity is undefined behavior.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::{Array, Vector};
    /// let arr = Array::from(1_u8..=3);
    /// let mut vec = Vector::with_cap(arr.size());
    /// for i in arr.into_iter() {
    ///     // SAFETY: We know that vec has enough capacity to store all elements in arr.
    ///     unsafe { vec.push_unchecked(i); }
    /// }
    /// assert_eq!(&*vec, &[1, 2, 3]);
    /// ```
    pub unsafe fn push_unchecked(&mut self, value: T) {
        // SAFETY: It is up to the caller to ensure that the Vector has enough capacity for this
        // push, leading to the pointer read being in bounds of the object.
        // TODO: ensure that size_of::<T>() * self.len can't overflow isize::MAX. Should wrap len+=1
        unsafe { self.arr.ptr.add(self.len).write(MaybeUninit::new(value)); }
        self.len += 1;
    }

    /// Pops the last value off the end of the Vector, returning an owned value if the Vector has
    /// length greater than 0.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
    /// let mut vec = Vector::from(0..5);
    /// for i in (0..vec.len()).rev() {
    ///     assert_eq!(vec.pop(), Some(i));
    /// }
    /// assert_eq!(vec.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            // Decrement len before getting.
            self.len -= 1;
            let value = unsafe {
                // SAFETY: len has just been decremented and is within the capacity of the Vector.
                // size_of::<T>() * self.len can't overflow isize::MAX, and all values < len are
                // initialized.
                // We are making a bitwise copy of the value on the heap and then forgetting that
                // the version on the heap exists, which is as close as we can get to moving the
                // value off of the heap.
                self.arr.ptr.add(self.len).read().assume_init()
            };
            Some(value)
        }
    }

    /// Inserts the provided value at the given index, growing and moving items as nessecary.
    /// 
    /// # Panics
    /// Panics if the provided index is out of bounds.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
    /// let mut vec = Vector::from(0..3);
    /// vec.insert(1, 100);
    /// vec.insert(1, 200);
    /// vec.insert(3, 300);
    /// assert_eq!(&*vec, &[0, 200, 100, 300, 1, 2]);
    /// ```
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

    /// Removes the element at the provided index, moving all following values to fill in the gap.
    /// 
    /// # Panics
    /// Panics if the provided index is out of bounds.
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Vector;
    /// let mut vec: Vector<_> = "Hello world!".chars().collect();
    /// assert_eq!(vec.remove(1), 'e');
    /// assert_eq!(vec.remove(4), ' ');
    /// assert_eq!(vec, dbg!("Hlloworld!".chars()).collect());
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        self.check_index(index);

        let mut next = MaybeUninit::uninit();
        // Iterate backwards to index.
        for i in (index..self.len).rev() {
            next = mem::replace(&mut self.arr[i], next);
        }

        self.len -= 1;
        // SAFETY: next contains the value which was previously located at index, which we've
        // already checked to be less than len and therefore initialised.
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

    // new_cap = len + extra
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
                unsafe {
                    // SAFETY: The pointer is nonnull, as well as properly aligned, initialized and
                    // ready to drop.
                    ptr::drop_in_place(
                        // SAFETY: count > isize::MAX is already guarded against and all possible
                        // values are within the allocated range of the Array.
                        ptr.add(i).as_ptr()
                    );
                }
                self.len -= 1;
            }
        }

        self.realloc_with_cap(ptr, old_cap, new_cap);
    }

    pub fn append(&mut self, other: Vector<T>) {
        self.extend_reserve(other.len);
        for item in other.into_iter() {
            self.extend_one(item);
        }
    }
}

impl<T> Vector<T> {
    pub(crate) fn realloc_with_cap(
        &mut self,
        ptr: NonNull<MaybeUninit<T>>,
        old_cap: usize,
        new_cap: usize
    ) {
        // I didn't think that handling zero-sized types would be quite so easy. Turns out the
        // solution is: just don't allocate anything. **tada**
        // ptr::read (and functions which rely on it) handle zero sized types for us, so as long as
        // we ensure that alloc and realloc are being used properly, we don't need to worry about
        // allocations at all.
        if size_of::<T>() == 0 { return; }

        let new_ptr = match (old_cap, new_cap) {
            (old, new) if old == new => {
                // The capacities are equal, do nothing there is no need to reallocate.
                return;
            },
            (0, _) => {
                // If the vec previously had a capacity of zero, we need a new allocation.
                let layout = Array::<MaybeUninit<T>>::make_layout(new_cap);
                
                let raw_ptr: *mut MaybeUninit<T> = unsafe {
                    // SAFETY: Layout will have non-zero size because both 0 capacity and zero-sized
                    // types are guarded against.
                    alloc::alloc(layout).cast()
                };

                NonNull::new(raw_ptr).unwrap_or_else(
                    || alloc::handle_alloc_error(layout)
                )
            },
            (_, 0) => {
                // If the new capacity is zero, we just need a dangling pointer.
                NonNull::dangling()
            },
            (_, _) => {
                // Otherwise, use realloc to handle moving or in-place size changing.
                let layout = Array::<MaybeUninit<T>>::make_layout(old_cap);
                
                // TODO: handle isize::MAX stuff everywhere
                let raw_ptr: *mut MaybeUninit<T> = unsafe {
                    // SAFETY: The same layout and allocator are used for the allocation, and the
                    // new size is > 0 and <= isize::MAX.
                    alloc::realloc(
                        ptr.as_ptr().cast(),
                        layout,
                        new_cap * size_of::<T>()
                    ).cast()
                };

                NonNull::new(raw_ptr).unwrap_or_else(
                    || alloc::handle_alloc_error(layout)
                )
            },
        };
        

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

    pub(crate) fn check_index(&self, index: usize) {
        assert!(
            index < self.len,
            "index {} out of bounds for collection with {} elements",
            index,
            self.len
        );
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
        unsafe { self.push_unchecked(item) }
    }
}

impl<T, I> From<I> for Vector<T>
where
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator
{
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let mut vec = Vector::with_cap(iter.len());

        for item in iter {
            // SAFETY: Vec has been created with the right capacity.
            unsafe { vec.push_unchecked(item); }
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
        // Call drop on all initialized values in place.
        for i in 0..self.len {
            unsafe { self.arr.ptr.add(i).as_mut().assume_init_drop(); }
        }
        
        // We don't need to handle the Array, because it contains only MaybeUninit values, which
        // do nothing when dropped. We know that everything important has already been dropped.
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(
                // Reinterpret *mut MaybeUninit<T> as *mut T for all values < len.
                self.arr.ptr.as_ptr().cast(),
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
                self.arr.ptr.as_ptr().cast(),
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

impl<T> From<Array<T>> for Vector<T> {
    fn from(value: Array<T>) -> Self {
        let len = value.size();
        Vector {
            // SAFETY: Array<MaybeUninit<T>> has the same layout as Array<T>.
            arr: unsafe { mem::transmute(value) },
            len
        }
    }
}

impl<T: PartialEq> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && &**self == &**other
    }
}

impl<T: Eq> Eq for Vector<T> {}