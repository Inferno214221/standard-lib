use std::borrow::{Borrow, BorrowMut};
use std::cmp;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::TrustedLen;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::slice;

use crate::collections::contiguous::Array;
use crate::util::error::IndexOutOfBounds;
use crate::util::result::ResultExtension;

const MIN_CAP: usize = 2;
const MAX_CAP: usize = isize::MAX as usize;

const GROWTH_FACTOR: usize = 2;

// TODO: Add try methods.

/// A variable size contiguous collection, based on [`Array<T>`].
///
/// # Time Complexity
/// For this analysis of time complexity, variables are defined as follows:
/// - `n`: The number of items in the Vector.
/// - `i`: The index of the item in question.
/// - `m`: The number of items in the second Vector.
///
/// | Method | Complexity |
/// |-|-|
/// | `get` | `O(1)` |
/// | `len` | `O(1)` |
/// | `push` | `O(1)`*, `O(n)` |
/// | `push_unchecked` | `O(1)` |
/// | `pop` | `O(1)` |
/// | `insert` | `O(n-i)` |
/// | `remove` | `O(n-i)` |
/// | `replace` | `O(1)` |
/// | `reserve` | `O(n)`**, `O(1)` |
/// | `shrink_to_fit` | `O(n)` |
/// | `adjust_cap` | `O(n)` |
/// | `append` | `O(n+m)` |
/// | `contains` | `O(n)` |
///
/// \* If the Vector doesn't have enough capacity for the new element, `push` will take `O(n)`.
///
/// \** If the Vector has enough capacity for the additional items already, `reserve` is `O(1)`.
pub struct Vector<T> {
    pub(crate) arr: Array<MaybeUninit<T>>,
    pub(crate) len: usize,
}

impl<T> Vector<T> {
    /// Creates a new Vector with length and capacity 0. Memory will be allocated when the capacity
    /// changes.
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let vec: Vector<u8> = Vector::new();
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.cap(), 0);
    /// ```
    pub fn new() -> Vector<T> {
        Vector {
            arr: Array::new(),
            len: 0,
        }
    }

    /// Creates a new Vector with capacity exactly equal to the provided value, allowing values to
    /// be added without reallocation.
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let mut vec: Vector<u8> = Vector::with_cap(5);
    /// assert_eq!(vec.cap(), 5);
    /// vec.extend([1_u8, 2, 3, 4, 5]);
    /// assert_eq!(vec.cap(), 5);
    /// ```
    pub fn with_cap(cap: usize) -> Vector<T> {
        Vector {
            arr: Array::new_uninit(cap),
            len: 0,
        }
    }

    /// Returns the length of the Vector.
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let vec = Vector::from_iter_sized(1_u8..=3);
    /// assert_eq!(vec.len(), 3);
    /// ```
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the Vector contains no elements.
    ///  
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let mut vec: Vector<u8> = Vector::new();
    /// assert!(vec.is_empty());
    /// vec.push(1);
    /// assert!(!vec.is_empty())
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the current capacity of the Vector. Unlike [`Vec`], the capacity is guaranteed to be
    /// exactly the value provided to any of the various capacity manipulation functions.
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let vec: Vector<u8> = Vector::with_cap(5);
    /// assert_eq!(vec.cap(), 5);
    /// ```
    pub const fn cap(&self) -> usize {
        self.arr.size()
    }

    /// Push the provided value onto the end of the Vector, increasing the capacity if required.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
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
    /// # use standard_lib::collections::contiguous::{Array, Vector};
    /// let arr = Array::from_iter_sized(1_u8..=3);
    /// let mut vec = Vector::with_cap(arr.size());
    /// for i in arr.into_iter() {
    ///     // SAFETY: We know that vec has enough capacity to store all elements in arr.
    ///     unsafe { vec.push_unchecked(i); }
    /// }
    /// assert_eq!(&*vec, &[1, 2, 3]);
    /// ```
    pub const unsafe fn push_unchecked(&mut self, value: T) {
        // SAFETY: It is up to the caller to ensure that the Vector has enough capacity for this
        // push, leading to the pointer read being in bounds of the object.
        unsafe { self.arr.ptr.add(self.len).write(MaybeUninit::new(value)); }
        self.len += 1;
    }

    /// Pops the last value off the end of the Vector, returning an owned value if the Vector has
    /// length greater than 0.
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let mut vec = Vector::from_iter_sized(0..5);
    /// for i in (0..vec.len()).rev() {
    ///     assert_eq!(vec.pop(), Some(i));
    /// }
    /// assert_eq!(vec.pop(), None);
    /// ```
    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            // Decrement len before getting.
            self.len -= 1;

            // SAFETY: len has just been decremented and is within the capacity of the Vector.
            // size_of::<T>() * self.len can't overflow isize::MAX, and all values < len are
            // initialized.
            // We are making a bitwise copy of the value on the heap and then forgetting that the
            // version on the heap exists, which is as close as we can get to actually moving the
            // value off of the heap.
            let value = unsafe {
                self.arr.ptr.add(self.len).read().assume_init()
            };
            Some(value)
        }
    }

    /// Inserts the provided value at the given index, growing and moving items as necessary.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    ///
    /// # Examples
    /// ```
    /// # use standard_lib::collections::contiguous::Vector;
    /// let mut vec = Vector::from_iter_sized(0..3);
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
    /// # use standard_lib::collections::contiguous::Vector;
    /// let mut vec: Vector<_> = "Hello world!".chars().collect();
    /// assert_eq!(vec.remove(1), 'e');
    /// assert_eq!(vec.remove(4), ' ');
    /// assert_eq!(vec, "Hlloworld!".chars().collect());
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
        // already checked to be less than len and therefore initialized.
        unsafe { next.assume_init() }
    }

    /// Replaces the element at the provided index with `new_value`, returning the old value.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub fn replace(&mut self, index: usize, new_value: T) -> T {
        self.check_index(index);

        // SAFETY: index is < len and all values < len are initialized.
        unsafe {
            mem::replace(
                &mut self.arr[index],
                MaybeUninit::new(new_value)
            ).assume_init()
        }
    }

    /// Ensures that the Vector has capacity to hold an additional `extra` elements. After invoking
    /// this method, the capacity will be >= len + extra.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub fn reserve(&mut self, extra: usize) {
        let new_cap = self.len.strict_add(extra);

        if new_cap < self.cap() { return; }

        self.realloc_with_cap(new_cap);
    }

    /// Shrinks the Vector so that its capacity is equal to its length.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub fn shrink_to_fit(&mut self) {
        self.realloc_with_cap(self.len);
    }

    /// Adjusts the capacity of the Vector to `new_cap`, dropping elements if required.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub fn adjust_cap(&mut self, new_cap: usize) {
        if new_cap < self.cap() {
            // Drop the values that are about to be deallocated.
            for i in new_cap..self.cap() {
                // SAFETY: The pointer is nonnull, as well as properly aligned, initialized and
                // ready to drop. count > isize::MAX is already guarded against and all possible
                // values are within the allocated range of the Array.
                unsafe {
                    ptr::drop_in_place(
                        self.arr.ptr.add(i).as_ptr()
                    );
                }
                self.len -= 1;
            }
        }

        self.realloc_with_cap(new_cap);
    }

    /// Appends all elements from `other` to self.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub fn append(&mut self, other: Vector<T>) {
        let initial_len = self.len;
        self.reserve(other.len);

        // SAFETY: self is valid from initial_len to initial_len + other.len and other is valid from
        // 0 to other.len. Both are properly aligned and don't overlap.
        unsafe {
            // Reduce iteration by copying one slice into the other.
            ptr::copy_nonoverlapping(
                self.arr.ptr.add(initial_len).as_ptr().cast_const(),
                other.arr.ptr.as_ptr(),
                other.len,
            );
        }

        self.len += other.len;

        // Forget about other because we have copied all values.
        mem::forget(other);
    }
    
    pub const fn into_parts(self) -> (NonNull<MaybeUninit<T>>, usize, usize) {
        let ret = (self.arr.ptr, self.len, self.arr.size);
        mem::forget(self);
        ret
    }

    pub const unsafe fn from_parts(
        ptr: NonNull<MaybeUninit<T>>,
        len: usize,
        cap: usize,
    ) -> Vector<T> {
        Vector {
            arr: unsafe { Array::from_parts(ptr, cap) },
            len,
        }
    }

    /// Creates an Vector from a type which implements [`IntoIterator`] and creates an
    /// [`ExactSizeIterator`].
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    pub fn from_iter_sized<I>(value: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator + TrustedLen
    {
        let iter = value.into_iter();
        let mut vec = Vector::with_cap(iter.len());

        for item in iter {
            // SAFETY: vec has been created with the right capacity.
            unsafe { vec.push_unchecked(item); }
        }

        vec
    }

    /// Reallocates the internal Array with the provided capacity.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub(crate) fn realloc_with_cap(&mut self, new_cap: usize) {
        self.arr.realloc(new_cap);
    }

    /// Grows the internal Array to allow for the insertion of additional elements. After calling
    /// this, the Vector can take at least one more element.
    ///
    /// # Panics
    /// Panics if the memory layout of the Vector would have a size that exceeds [`isize::MAX`].
    pub(crate) fn grow(&mut self) {
        // SAFETY: old_cap < isize::MAX, so old_cap * 2 can't overflow. Can still exceed isize::MAX.
        let mut new_cap = cmp::max(self.cap() * GROWTH_FACTOR, MIN_CAP);

        // If we would grow past maximum capacity, instead use the maximum if it represents growth.
        if (new_cap * size_of::<T>() > MAX_CAP) && (MAX_CAP > self.cap() * size_of::<T>()) {
            new_cap = MAX_CAP;
        }

        self.realloc_with_cap(new_cap);
    }

    /// Checks that the provided index is within the bounds of self.
    ///
    /// # Panics
    /// Panics if the provided index is out of bounds.
    pub(crate) fn check_index(&self, index: usize) {
        if index >= self.len {
            Err(IndexOutOfBounds {
                index,
                len: self.len
            }).throw()
        }
    }
}

impl<T> Extend<T> for Vector<T> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        for item in iter {
            self.push(item);
        }
    }

    fn extend_one(&mut self, item: T) {
        self.push(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    unsafe fn extend_one_unchecked(&mut self, item: T)
    where
        Self: Sized,
    {
        // SAFETY: extend_reserve is implemented correctly, so all other safety requirements are the
        // responsibility of the caller.
        unsafe { self.push_unchecked(item); }
    }
}

// impl<T, I> From<I> for Vector<T>
// where
//     I: Iterator<Item = T> + ExactSizeIterator + TrustedLen,
// {
//     fn from(value: I) -> Self {
//         let iter = value.into_iter();
//         let mut vec = Vector::with_cap(iter.len());

//         for item in iter {
//             // SAFETY: vec has been created with the right capacity.
//             unsafe { vec.push_unchecked(item); }
//         }

//         vec
//     }
// }

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(value: I) -> Self {
        let iter = value.into_iter();
        let mut vec = Vector::with_cap(iter.size_hint().0);

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
            // SAFETY: All values less than len are initialized and safe to drop.
            unsafe { self.arr.ptr.add(i).as_mut().assume_init_drop(); }
        }

        // Implicitly drop self.arr, containing only MaybeUninit values without a no-op drop.
        // Doing so also deallocates the owned memory.
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: Vector is valid as a slice for len values, which are all initialized. The pointer
        // is nonnull, properly aligned and the range entirely contained within this Vector.
        // The borrow checker enforces that self isn't mutated due to this function taking a &self.
        // The total size is < isize::MAX as the result of being a valid Vector.
        unsafe {
            slice::from_raw_parts(
                // Reinterpret *mut MaybeUninit<T> as *mut T for all values < len.
                self.arr.ptr.as_ptr().cast(),
                self.len,
            )
        }
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Vector is valid as a slice for len values, which are all initialized. The pointer
        // is nonnull, properly aligned and the range entirely contained within this Vector.
        // The borrow checker enforces that self isn't accessed due to this function taking a &self.
        // The total size is < isize::MAX as the result of being a valid Vector.
        unsafe {
            slice::from_raw_parts_mut(
                // Reinterpret *mut MaybeUninit<T> as *mut T for all values < len.
                self.arr.ptr.as_ptr().cast(),
                self.len,
            )
        }
    }
}

impl<T> AsRef<[T]> for Vector<T> {
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<T> AsMut<[T]> for Vector<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> Borrow<[T]> for Vector<T> {
    fn borrow(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T> BorrowMut<[T]> for Vector<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

// SAFETY: Vectors, when used safely rely on unique pointers and are therefore safe for Send when T:
// Send.
unsafe impl<T: Send> Send for Vector<T> {}
// SAFETY: Vector's safe API obeys all rules of the borrow checker, so no interior mutability
// occurs. This means that Vector<T> can safely implement Sync when T: Sync.
unsafe impl<T: Sync> Sync for Vector<T> {}

impl<T: Clone> Clone for Vector<T> {
    fn clone(&self) -> Self {
        let mut vec = Self::with_cap(self.cap());

        for value in self.iter() {
            vec.push(value.clone());
        }

        vec
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
            arr: value.forget_init(),
            len,
        }
    }
}

impl<T> From<Vector<T>> for Vec<T> {
    fn from(value: Vector<T>) -> Self {
        let (ptr, len, cap) = value.into_parts();
        unsafe { Vec::from_parts(ptr.cast(), len, cap) }
    }
}

impl<T> From<Vec<T>> for Vector<T> {
    fn from(value: Vec<T>) -> Self {
        let (ptr, len, cap) = value.into_parts();
        unsafe { Vector::from_parts(ptr.cast(), len, cap) }
    }
}

impl From<String> for Vector<u8> {
    fn from(value: String) -> Self {
        Vec::from(value).into()
    }
}

impl TryFrom<Vector<u8>> for String {
    type Error = <String as TryFrom<Vec<u8>>>::Error;
    
    fn try_from(value: Vector<u8>) -> Result<Self, Self::Error> {
        Vec::from(value).try_into()
    }
}

impl<T: PartialEq> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for Vector<T> {}

impl<T: Hash> Hash for Vector<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vector")
            .field_with("contents", |f| f.debug_list().entries(self.iter()).finish())
            .field("len", &self.len)
            .field("cap", &self.cap())
            .finish()
    }
}

impl<T: Debug> Display for Vector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "!")?;
        f.debug_list().entries(self.iter()).finish()
    }
}
