use std::alloc::{self, Layout};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{self, Debug, Display, Formatter};
use std::iter::TrustedLen;
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::slice;

const MAX_SIZE: usize = isize::MAX as usize;

/// An implementation of an array that is sized at runtime. Similar to a [`Box<[T]>`](Box<T>).
///
/// # Time Complexity
/// For this analysis of time complexity, variables are defined as follows:
/// - `n`: The number of items in the Array.
/// - `i`: The index of the item in question.
///
/// | Method | Complexity |
/// |-|-|
/// | `get` | `O(1)` |
/// | `size` | `O(1)` |
/// | `realloc` | `O(n)`*, `O(1)` |
/// | `contains` | `O(n)` |
///
/// \* It might be possible to get an `O(1)` reallocation, but I don't believe it is very likely.
pub struct Array<T> {
    pub(crate) ptr: NonNull<T>,
    pub(crate) size: usize,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Array<T> {
    /// Returns the size of the Array.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
    /// let arr = Array::from([1, 2, 3].into_iter());
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
    /// # use rust_basic_types::contiguous::Array;
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
    /// Panics if memory layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
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
            _phantom: PhantomData,
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
    pub const fn into_parts(self) -> (NonNull<T>, usize) {
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
    /// - `size` needs to be less than or equal to [`isize::MAX`] / `size_of::<T>()`.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
    /// let arr = Array::from([1_u8, 2, 3].into_iter());
    /// let (ptr, size) = arr.into_parts();
    /// assert_eq!(
    ///     unsafe { Array::from_parts(ptr, size) },
    ///     Array::from([1, 2, 3].into_iter())
    /// );
    /// ```
    pub const unsafe fn from_parts(ptr: NonNull<T>, size: usize) -> Array<T> {
        Array {
            ptr,
            size,
            _phantom: PhantomData,
        }
    }

    /// Interprets self as an `Array<MaybeUninit<T>>`. Although it may not seem very useful by
    /// itself, this method acts as a counterpart to [`Array::assume_init`] and allows
    /// [`Array::realloc`] to be called on a previously initialized Array.
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
    /// # use std::mem::MaybeUninit;
    /// let mut arr = Array::from([1_u8, 2, 3].into_iter());
    /// let mut new_arr = arr.forget_init();
    ///
    /// new_arr.realloc(4);
    /// new_arr[3] = MaybeUninit::new(4);
    ///
    /// // SAFETY: All values in new_arr are now initialized.
    /// arr = unsafe { new_arr.assume_init() };
    ///
    /// assert_eq!(&*arr, &[1, 2, 3, 4]);
    /// ```
    pub fn forget_init(self) -> Array<MaybeUninit<T>> {
        // SAFETY: Array<T> has the same layout as Array<MaybeUninit<T>>.
        unsafe { mem::transmute::<Array<T>, Array<MaybeUninit<T>>>(self) }
    }
}

impl<T> Array<T> {
    /// A helper function to create a [`Layout`] for use during allocation, containing `size` number
    /// of elements of type `T`.
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    pub(crate) fn make_layout(size: usize) -> Layout {
        Layout::array::<T>(size).expect("Capacity overflow!")
    }

    /// A helper function to create a [`NonNull`] for the provided [`Layout`]. Returns a dangling
    /// pointer for a zero-sized layout.
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
                unsafe { alloc::alloc(layout).cast() }
            ).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        }
    }

    pub(crate) const unsafe fn clone_shallow(&mut self) -> Array<T> {
        // SAFETY: There are no safety guarantees here, responsibility it passed to the caller.
        unsafe { Array::from_parts(self.ptr, self.size) }
    }
}

impl<T: Copy> Array<T> {
    /// Creates a new `Array<T>` with `count` copies of `item`.
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
    /// let arr = Array::repeat_item(5, 3);
    /// assert_eq!(arr.size(), 3);
    /// assert_eq!(&*arr, &[5, 5, 5]);
    /// ```
    pub fn repeat_item(item: T, count: usize) -> Array<T> {
        let arr = Self::new_uninit(count);

        for i in 0..count {
            // SAFETY: size > isize::MAX / size_of::<T>() is already guarded against and all
            // possible values are within the allocated range of the Array.
            unsafe {
                arr.ptr.add(i).write(MaybeUninit::new(item))
            }
        }

        // SAFETY: All values are initialized with a copy of item.
        unsafe { arr.assume_init() }
    }

    /// Reallocate self with `new_size`, filling any extra elements with a copy of `item`.
    ///
    /// # Panics
    /// Panics if the memory layout of the new allocation would have a size that exceeds
    /// [`isize::MAX`]. (`new_size * size_of::<T>() > isize::MAX`)
    pub fn realloc_with_copy(&mut self, item: T, new_size: usize) {
        // SAFETY: We use a shallow clone here to allow us to change the type of the Array without
        // moving it out from behind a mutable reference. Neither of the Arrays are dropped and self
        // isn't used for the entire lifetime of the clone, except to access the original size
        // without copying.
        let mut wip_arr = unsafe { self.clone_shallow().forget_init() };
        wip_arr.realloc(new_size);

        for i in self.size..wip_arr.size {
            // SAFETY: size > isize::MAX / size_of::<T>() is already guarded against and all
            // possible values are within the allocated range of the new Array.
            unsafe {
                wip_arr.ptr.add(i).write(MaybeUninit::new(item))
            }
        }

        // Forget the old value to prevent a double free.
        mem::forget(mem::replace(
            self,
            // SAFETY: wip_arr is now initialized with copies of item.
            unsafe { wip_arr.assume_init() }
        ));
    }
}

impl<T: Default> Array<T> {
    /// Creates a new `Array<T>` by repeating the default value of `T` `count` times.
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    pub fn repeat_default(count: usize) -> Array<T> {
        let arr = Self::new_uninit(count);

        for i in 0..count {
            // SAFETY: size > isize::MAX / size_of::<T>() is already guarded against and all
            // possible values are within the allocated range of the Array.
            unsafe {
                arr.ptr.add(i).write(MaybeUninit::new(T::default()))
            }
        }

        // SAFETY: All values are initialized with the default value for T.
        unsafe { arr.assume_init() }
    }

    /// Reallocate self with `new_size`, filling any extra elements with the default value of `T`.
    ///
    /// # Panics
    /// Panics if the memory layout of the new allocation would have a size that exceeds
    /// [`isize::MAX`]. (`new_size * size_of::<T>() > isize::MAX`)
    pub fn realloc_with_default(&mut self, new_size: usize) {
        // SAFETY: We use a shallow clone here to allow us to change the type of the Array without
        // moving it out from behind a mutable reference. Neither of the Arrays are dropped and self
        // isn't used for the entire lifetime of the clone, except to access the original size
        // without copying.
        let mut wip_arr = unsafe { self.clone_shallow().forget_init() };
        wip_arr.realloc(new_size);

        for i in self.size..wip_arr.size {
            // SAFETY: size > isize::MAX / size_of::<T>() is already guarded against and all
            // possible values are within the allocated range of the Array.
            unsafe {
                wip_arr.ptr.add(i).write(MaybeUninit::new(T::default()))
            }
        }

        // Forget the old value to prevent a double free.
        mem::forget(mem::replace(
            self,
            // SAFETY: wip_arr is now initialized with the default value for T.
            unsafe { wip_arr.assume_init() }
        ));
    }
}

impl<T, I> From<I> for Array<T>
where
    I: Iterator<Item = T> + ExactSizeIterator + TrustedLen,
{
    /// Creates an Array from a type which implements [`IntoIterator`] and creates an
    /// [`ExactSizeIterator`].
    ///
    /// # Panics
    /// Panics if memory layout size exceeds [`isize::MAX`].
    ///
    /// # Examples
    /// ```
    /// # use rust_basic_types::contiguous::Array;
    /// let arr = Array::from([1, 2, 3].into_iter());
    /// assert_eq!(&*arr, [1, 2, 3]);
    /// ```
    fn from(iter: I) -> Self {
        let size = iter.len();
        let arr = Self::new_uninit(size);

        for (index, item) in iter.enumerate() {
            // SAFETY: size > isize::MAX / size_of::<T>() is already guarded against and all
            // possible values are within the allocated range of the Array.
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
    /// # use rust_basic_types::contiguous::Array;
    /// # use std::mem::MaybeUninit;
    /// let mut arr = Array::new_uninit(5);
    /// for i in 0..5 {
    ///     arr[i] = MaybeUninit::new(i);
    /// }
    /// assert_eq!(&*unsafe { arr.assume_init() }, &[0, 1, 2, 3, 4]);
    /// ```
    pub unsafe fn assume_init(self) -> Array<T> {
        // SAFETY: There are no safety guarantees here, responsibility it passed to the caller.
        unsafe { self.transpose().assume_init() }
    }

    /// Reallocate the Array to have size equal to new_size, with new locations uninitialized.
    /// Several checks are performed first to ensure that an allocation is actually required.
    ///
    /// # Panics
    /// Panics if the memory layout of the new allocation would have a size that exceeds
    /// [`isize::MAX`]. (`new_size * size_of::<T>() > isize::MAX`)
    ///
    /// # Examples
    /// TODO
    pub fn realloc(&mut self, new_size: usize) {
        let new_ptr = match (self.size, new_size) {
            (_, _) if size_of::<T>() == 0 => {
                // I didn't think that handling zero-sized types would be quite so easy. Turns out
                // the solution is: just don't allocate anything. **tada**
                // ptr::read (and functions which rely on it) handle zero sized types for us, so as
                // long as we ensure that alloc and realloc are being used properly, we don't need
                // to worry about allocations at all.

                // We still need to return the existing dangling pointer so that the Array's size
                // can be updated.
                self.ptr
            },
            (old, new) if old == new => {
                // The capacities are equal, do nothing there is no need to reallocate.
                // SAFETY: Array<T> has the same layout as Array<MaybeUninit<T>>.
                return;
            },
            (0, _) => {
                // If the Array previously had a capacity of zero, we need a new allocation.
                let layout = Array::<MaybeUninit<T>>::make_layout(new_size);

                // SAFETY: Layout will have non-zero size because both 0 capacity and zero-sized
                // types are guarded against.
                let raw_ptr: *mut MaybeUninit<T> = unsafe {
                    alloc::alloc(layout).cast()
                };

                NonNull::new(raw_ptr).unwrap_or_else(
                    || alloc::handle_alloc_error(layout)
                )
            },
            (_, 0) => {
                // If the new size is zero, we just need a dangling pointer.
                NonNull::dangling()
            },
            (_, _) => {
                // Otherwise, use realloc to handle moving or in-place size changing.
                let layout = Array::<MaybeUninit<T>>::make_layout(self.size);

                if new_size * size_of::<T>() > MAX_SIZE {
                    panic!("Capacity overflow!")
                }

                // SAFETY: The same layout and allocator are used for the allocation, and the new
                // layout size is > 0 and <= isize::MAX.
                let raw_ptr: *mut MaybeUninit<T> = unsafe {
                    alloc::realloc(
                        self.ptr.as_ptr().cast(),
                        layout,
                        new_size * size_of::<T>()
                    ).cast()
                };

                NonNull::new(raw_ptr).unwrap_or_else(
                    || alloc::handle_alloc_error(layout)
                )
            },
        };

        self.ptr = new_ptr;
        self.size = new_size;
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
            // SAFETY: The pointer is nonnull, as well as properly aligned, initialized and
            // ready to drop.
            // SAFETY: count > isize::MAX / size_of::<T>() is already guarded against and
            // all possible values are within the allocated range of the Array.
            unsafe {
                ptr::drop_in_place(self.ptr.add(i).as_ptr());
            }
        }

        if layout.size() != 0 {
            // SAFETY: ptr is always allocated in the global allocator and layout is the same as
            // when allocated. Zero-sized layouts aren't allocated and are guarded against
            // deallocation.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast(), layout)
            }
        }
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: The held data uses Layout::array(size) and is therefore valid and properly
        // aligned for (size * mem::size_of::<T>()) bytes. Data is properly initialized and has a
        // length no greater than isize::MAX. Array's safe API doesn't provide access to raw
        // pointers, so the borrow checker prevents mutation throughout 'a.
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.size)
        }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The held data uses Layout::array(size) and is therefore valid and properly
        // aligned for (size * mem::size_of::<T>()) bytes. Data is properly initialized and has a
        // length no greater than isize::MAX. Array's safe API doesn't provide access to raw
        // pointers, so the borrow checker prevents access throughout 'a.
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size)
        }
    }
}

impl<T> AsRef<[T]> for Array<T> {
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<T> AsMut<[T]> for Array<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> Borrow<[T]> for Array<T> {
    fn borrow(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T> BorrowMut<[T]> for Array<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

// SAFETY: Arrays, when used safely rely on unique pointers and are therefore safe for Send when T:
// Send.
unsafe impl<T: Send> Send for Array<T> {}
// SAFETY: Array's safe API obeys all rules of the borrow checker, so no interior mutability occurs.
// This means that Array<T> can safely implement Sync when T: Sync.
unsafe impl<T: Sync> Sync for Array<T> {}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array::from(self.iter().cloned())
    }
}

impl<T: PartialEq> PartialEq for Array<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for Array<T> {}

impl<T: Debug> Debug for Array<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Array")
            .field_with("contents", |f| f.debug_list().entries(self.iter()).finish())
            .field("size", &self.size)
            .finish()
    }
}

impl<T: Debug> Display for Array<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
