use std::alloc;
use std::fmt::{self, Debug, Formatter};
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use super::Array;

pub struct Vector<T> {
    pub(crate) arr: Array<MaybeUninit<T>>,
    pub(crate) len: usize
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

impl<T> Vector<T> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn cap(&self) -> usize {
        self.arr.size()
    }

    pub fn new() -> Vector<T> {
        Vector {
            arr: Array::new(),
            len: 0
        }
    }

    pub fn with_cap(cap: usize) -> Vector<T> {
        Vector {
            arr: Array::new_uninit(cap),
            len: 0
        }
    }
    
    fn grow(&mut self) {
        let ptr = self.arr.ptr;
        let cap = self.cap();
        let new_cap = if cap == 0 { 1 } else { cap * 2 };
        
        let layout = Array::<MaybeUninit<T>>::make_layout(cap);

        let new_ptr = NonNull::new(unsafe {
            if cap == 0 {
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
        }).unwrap();

        // Prevent a double free by forgetting the old value of self.arr.
        mem::forget(mem::replace(
            &mut self.arr,
            unsafe { Array::from_parts(new_ptr, new_cap) }
        ));
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap() {
            self.grow()
        }
        self.arr[self.len] = MaybeUninit::new(value);
        self.len += 1
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

    fn check_index(&self, index: usize) {
        if index >= self.len {
            panic!("Index {} out of bounds for Vector with len {}", index, self.len);
        }
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

    pub fn swap_value(&mut self, index: usize, new_value: T) -> T {
        self.check_index(index);

        unsafe { mem::replace(
            &mut self.arr[index],
            MaybeUninit::new(new_value)
        ).assume_init() }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn reserve(&mut self, extra: usize) {
        todo!()
    }

    pub fn grow_to(&mut self, size: usize) {
        todo!()
    }

    pub fn as_array(&self) -> Array<T> {
        todo!()
    }

    pub fn shrink_to_fit(&mut self) {
        todo!()
    }

    pub fn shrink_to(&mut self, size: usize) {
        todo!()
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
            .field("cap", &self.cap()).finish()
    }
}