use std::alloc::{self, Layout};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

/// An implementation of an array that is sized at runtime. Similar to a [`Box<[T]>`](Box<T>).
pub struct Array<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) size: usize,
    _phantom: PhantomData<T>
}

impl<T> Array<T> {
    /// Returns the size of the Array.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// let arr = Array::from([1, 2, 3]);
    /// assert_eq!(arr.size(), 3);
    /// ```
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Creates a new Array with size 0.
    ///
    /// This method isn't very helpful in most cases because the size remains zero after
    /// initialization. See [`Array::new_uninit`] or [`Array::from`] for preferred methods of
    /// initialization.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// let arr: Array<u8> = Array::new();
    /// assert_eq!(arr.size(), 0);
    /// assert_eq!(&*arr, &[]);
    /// ```
    pub fn new() -> Array<T> {
        // SAFETY: There are no values, so they are all initialized.
        unsafe { Self::new_uninit(0).assume_init() }
    }

    /// Creates a new Array of [`MaybeUninit<T>`] with the provided `size`. All values are
    /// uninitialized.
    ///
    /// # Panics
    /// Panics if layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// # use std::mem::MaybeUninit;
    /// let arr: Array<MaybeUninit<u8>> = Array::new_uninit(5);
    /// assert_eq!(arr.size(), 5);
    /// ```
    pub fn new_uninit(size: usize) -> Array<MaybeUninit<T>> {
        let layout = Array::<MaybeUninit<T>>::make_layout(size);
        let ptr = Array::<MaybeUninit<T>>::make_ptr(layout);

        Array {
            ptr,
            size,
            _phantom: PhantomData
        }
    }

    /// Decomposes an `Array<T>` into its raw components, a [`NonNull<T>`] pointer to the contained
    /// data and a [`usize`] representing the size.
    /// 
    /// Returns the pointer to the underlying data and the number of elements in the Array.
    /// 
    /// # Safety
    /// 
    /// After calling this function, the caller is responsible for the safety of the allocated data.
    /// The parts can be used to reconstruct an Array with [`Array::from_parts`], allowing it to be
    /// used again and dropped normally.
    /// 
    /// # Examples
    /// See [`Array::from_parts`].
    pub fn into_parts(self) -> (NonNull<T>, usize) {
        let ret = (self.ptr, self.size);
        mem::forget(self);
        ret
    }

    /// Creates an `Array<T>` from its raw components, a [`NonNull<T>`] pointer to the contained
    /// data and a [`usize`] representing the size.
    /// 
    /// # Safety
    /// 
    /// This is extremely unsafe, nothing is checked during construction.
    /// 
    /// For the produced value to be valid:
    /// - `ptr` needs to be a currently and correctly allocated pointer within the global allocator.
    /// - `ptr` needs to refer to `size` properly initialized values of `T`.
    /// - `size` needs to be less than [`isize::MAX`].
    /// 
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// let arr = Array::from([1, 2, 3]);
    /// let (ptr, size) = arr.into_parts();
    /// assert_eq!(
    ///     unsafe { Array::from_parts(ptr, size) },
    ///     Array::from([1, 2, 3])
    /// );
    /// ```
    pub unsafe fn from_parts(ptr: NonNull<T>, size: usize) -> Array<T> {
        Array {
            ptr,
            size,
            _phantom: PhantomData
        }
    }
}

impl<T> Array<T> {
    /// A helper function to create a [`Layout`] for use during allocation, containing `size` number
    /// of elements of type `T`.
    ///
    /// # Panics
    /// Panics if layout size exceeds [`isize::MAX`].
    pub(crate) fn make_layout(size: usize) -> Layout {
        Layout::array::<T>(size).expect("Capacity overflow!")
    }

    /// A helper function to create a [`NonNull`] for the provided [`Layout`]. Returns
    /// [`NonNull::dangling()`] for a zero-sized layout.
    ///
    /// # Errors
    /// In the event of an allocation error, this method calls [`alloc::handle_alloc_error`] as
    /// recommended, to avoid new allocations rather than panicking.
    pub(crate) fn make_ptr(layout: Layout) -> NonNull<T> {
        if layout.size() == 0 {
            NonNull::dangling()
        } else {
            NonNull::new(
                // SAFETY: Zero-sized layouts have been guarded against.
                unsafe { alloc::alloc(layout) as *mut T }
            ).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        }
    }
}

impl<T: Copy> Array<T> {
    /// Creates a new `Array<T>` with `count` copies of `item`.
    ///
    /// # Panics
    /// Panics if layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// let arr = Array::repeat(5, 3);
    /// assert_eq!(arr.size(), 3);
    /// assert_eq!(&*arr, &[5, 5, 5]);
    /// ```
    pub fn repeat(item: T, count: usize) -> Array<T> {
        let arr = Self::new_uninit(count);

        for i in 0..count {
            // SAFETY: size > isize::MAX is already guarded against and all possible values are
            // within the allocated range of the Array.
            unsafe {
                arr.ptr.add(i).write(MaybeUninit::new(item))
            }
        }

        // SAFETY: All values are initialized.
        unsafe { arr.assume_init() }
    }
}

impl<T, I> From<I> for Array<T> where I: IntoIterator<Item = T>, I::IntoIter: ExactSizeIterator {
    /// Creates an Array from a type which implements [`IntoIterator`] and creates an
    /// [`ExactSizeIterator`].
    ///
    /// # Panics
    /// Panics if layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// let arr = Array::from([1, 2, 3]);
    /// assert_eq!(&*arr, [1, 2, 3]);
    /// ```
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let size = iter.len();
        let arr = Self::new_uninit(size);

        for (index, item) in iter.enumerate() {
            // SAFETY: size > isize::MAX is already guarded against and all possible values are
            // within the allocated range of the Array.
            unsafe {
                arr.ptr.add(index).write(MaybeUninit::new(item))
            }
        }

        // SAFETY: All values are initialized.
        unsafe { arr.assume_init() }
    }
}

impl<T> Array<MaybeUninit<T>> {
    /// Converts a `Array<MaybeUninit<T>>` to `MaybeUninit<Array<T>>`.
    pub fn transpose(self) -> MaybeUninit<Array<T>> {
        // SAFETY: Array<MaybeUninit<T>> has the same layout as MaybeUninit<Array<T>>.
        unsafe { mem::transmute(self) }
    }

    /// Assume that all values of an `Array<MaybeUninit<T>>` are initialized.
    ///
    /// # Safety
    /// It is up to the caller to guarantee that the Array is properly initialized. Failing to do so
    /// is undefined behavior.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::Array;
    /// # use std::mem::MaybeUninit;
    /// let mut arr = Array::new_uninit(5);
    /// for i in 0..5 {
    ///     arr[i] = MaybeUninit::new(i);
    /// }
    /// assert_eq!(&*unsafe { arr.assume_init() }, &[0, 1, 2, 3, 4]);
    /// ```
    pub unsafe fn assume_init(self) -> Array<T> {
        unsafe { self.transpose().assume_init() }
    }
}

impl<T> Default for Array<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        let layout = Array::<T>::make_layout(self.size);

        for i in 0..self.size {
            drop(
                // SAFETY: count > isize::MAX is already guarded against and all possible values are
                // within the allocated range of the Array.
                unsafe { self.ptr.add(i).read() }
            );
        }

        if layout.size() != 0 {
            // SAFETY: ptr is always allocated in the global allocator and layout is the same as
            // when allocated. Zero-sized layouts aren't allocated and are guarded against
            // deallocation.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout)
            }
        }
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: The held data uses Layout::array(size) and is therefore valid and properly
        // aligned for (size * mem::size_of::<T>()) bytes. Data is properly initialized and has a
        // length no greater than isize::MAX.
        // Mutation throughout 'a is guaranteed by the compiler.
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.size)
        }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The held data uses Layout::array(size) and is therefore valid and properly
        // aligned for (size * mem::size_of::<T>()) bytes. Data is properly initialized and has a
        // length no greater than isize::MAX.
        // Accessing throughout 'a is guaranteed by the compiler.
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size)
        }
    }
}

impl<T: Debug> Debug for Array<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Array")
            .field("contents", &&**self)
            .field("size", &self.size)
            .finish()
    }
}

unsafe impl<T: Send> Send for Array<T> {}
unsafe impl<T: Sync> Sync for Array<T> {}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array::from(self.iter().map(|i| i.clone()))
    }
}

impl<T: PartialEq> PartialEq for Array<T> {
    fn eq(&self, other: &Self) -> bool {
        &**self == &**other
    }
}

impl<T: Eq> Eq for Array<T> {}