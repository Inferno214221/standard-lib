use std::{array::TryFromSliceError, mem::{self, MaybeUninit}, ops::{Index, IndexMut}};

use super::{Iter, IterMut};

const MAX_SIZE: usize = isize::MAX as usize;

const fn check_size(n: usize) {
    assert!(n <= MAX_SIZE, "N exceeds maximum size!");
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircStack<T, const N: usize> {
    pub(crate) buffer: [T; N],
    pub(crate) last: usize,
}

pub(crate) const fn increment<const N: usize>(index: usize) -> usize {
    // index + 1 <= MAX_SIZE because index < N <= MAX_SIZE. Can't overflow.
    (index + 1) % N
}

pub(crate) const fn sub_wrapping<const N: usize>(index: usize, diff: usize) -> usize {
    if index >= N {
        panic!("TODO")
    }

    unsafe {
        // SAFETY: index < N <= isize::MAX implies that self.index - index can't be less than
        // -isize::MAX, which is greater than isize::MIN.
        index.checked_signed_diff(diff).unwrap_unchecked()
    }.rem_euclid(N as isize) as usize
}

impl<T, const N: usize> CircStack<T, N> {
    pub(crate) const fn increment(&mut self) {
        self.last = increment::<N>(self.last)
    }

    pub(crate) const fn decrement(&mut self) {
        self.last = sub_wrapping::<N>(self.last, 1);
    }

    pub(crate) const fn translate_index(&self, index: usize) -> usize {
        sub_wrapping::<N>(self.last, index)
    }

    pub const fn new(buffer: [T; N], index: usize) -> CircStack<T, N> {
        const { check_size(N) };

        CircStack {
            buffer,
            last: index
        }
    }

    pub const fn new_uninit() -> CircStack<MaybeUninit<T>, N> {
        const { check_size(N) };

        CircStack {
            buffer: [const { MaybeUninit::uninit() }; N],
            last: 0
        }
    }

    pub fn push(&mut self, value: T) {
        self.increment();
        self.buffer[self.last] = value;
    }

    pub const fn pop_with_replacement(&mut self, replacement: T) -> T {
        let value = mem::replace(&mut self.buffer[self.last], replacement);
        self.decrement();
        value
    }

    pub const fn as_array(&self) -> &[T; N] {
        &self.buffer
    }

    pub const fn as_array_mut(&mut self) -> &mut [T; N] {
        &mut self.buffer
    }

    pub fn forget_init(self) -> CircStack<MaybeUninit<T>, N> {
        let CircStack { buffer, last } = self;

        CircStack {
            buffer: buffer.map(MaybeUninit::new),
            last
        }
    }

    // TODO: Verify this does what it should
    pub const fn rotate(&mut self, offset: isize) {
        if offset >= N as isize || offset <= -(N as isize) {
            panic!("TODO")
        }

        self.last = (self.last as isize + offset).rem_euclid(N as isize) as usize;
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T, N> {
        self.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, T, N> {
        self.into_iter()
    }
}

impl<T: Copy, const N: usize> CircStack<T, N> {
    pub const fn repeat_item(item: T) -> CircStack<T, N> {
        const { check_size(N) };

        CircStack {
            buffer: [item; N],
            last: 0,
        }
    }

    pub const fn pop_copy(&mut self) -> T {
        let value = self.buffer[self.last];
        self.decrement();
        value
    }

    pub const fn last(&self) -> &T {
        &self.buffer[self.last]
    }
}

impl<T: Default, const N: usize> CircStack<T, N> {
    pub fn pop_with_default(&mut self) -> T {
        self.pop_with_replacement(T::default())
    }
}

impl<T, const N: usize> CircStack<MaybeUninit<T>, N> {
    pub const fn transpose(self) -> MaybeUninit<CircStack<T, N>> {
        // Unfortunately, due to the variable size of the input and output, we have to copy here,
        // and avoid calling drop on the original.

        unsafe { mem::transmute_copy(&MaybeUninit::new(self)) }
    }

    pub const fn transpose_mut(&mut self) -> &mut MaybeUninit<CircStack<T, N>> {
        unsafe { mem::transmute(self) }
    }

    pub const fn transpose_ref(&self) -> &MaybeUninit<CircStack<T, N>> {
        unsafe { mem::transmute(self) }
    }

    pub const unsafe fn assume_init(self) -> CircStack<T, N> {
        unsafe { self.transpose().assume_init() }
    }

    pub const unsafe fn assume_init_mut(&mut self) -> &mut CircStack<T, N> {
        unsafe { self.transpose_mut().assume_init_mut() }
    }

    pub const unsafe fn assume_init_ref(&self) -> &CircStack<T, N> {
        unsafe { self.transpose_ref().assume_init_ref() }
    }
}

impl<T, const N: usize> CircStack<Option<T>, N> {
    pub fn transpose(self) -> Option<CircStack<T, N>> {
        let CircStack { buffer, last: index } = self;
        Some(CircStack {
            buffer: buffer.transpose()?,
            last: index,
        })
    }
}

impl<T: Default, const N: usize> Default for CircStack<T, N> {
    fn default() -> CircStack<T, N> {
        const { check_size(N) };

        CircStack {
            buffer: [(); N].map(|_| T::default()),
            last: 0,
        }
    }
}

impl<T: Copy, const N: usize> TryFrom<&[T]> for CircStack<T, N> {
    type Error = TryFromSliceError;

    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        const { check_size(N) };

        Ok(CircStack {
            buffer: <[T; N]>::try_from(value)?,
            last: 0,
        })
    }
}

impl<T, const N: usize> From<[T; N]> for CircStack<T, N> {
    fn from(buffer: [T; N]) -> Self {
        const { check_size(N) };

        CircStack {
            buffer,
            last: 0,
        }
    }
}

impl<T, const N: usize> Index<usize> for CircStack<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[self.translate_index(index)]
    }
}

impl<T, const N: usize> IndexMut<usize> for CircStack<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[self.translate_index(index)]
    }
}