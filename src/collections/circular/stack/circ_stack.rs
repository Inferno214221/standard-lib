use std::{array::TryFromSliceError, mem::{self, MaybeUninit}, ops::{Index, IndexMut}};

use super::{Iter, IterMut};

const MAX_SIZE: usize = isize::MAX as usize;

const fn check_size(n: usize) {
    assert!(n <= MAX_SIZE, "N exceeds maximum size!");
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircStack<T, const N: usize> {
    pub(crate) buffer: [T; N],
    pub(crate) tail: usize,
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
        self.tail = increment::<N>(self.tail)
    }

    pub(crate) const fn decrement(&mut self) {
        self.tail = sub_wrapping::<N>(self.tail, 1);
    }

    pub(crate) const fn translate_index(&self, index: usize) -> usize {
        sub_wrapping::<N>(self.tail, index)
    }

    pub const fn new(buffer: [T; N], tail: usize) -> CircStack<T, N> {
        const { check_size(N) };

        CircStack {
            buffer,
            tail
        }
    }

    pub const fn new_uninit() -> CircStack<MaybeUninit<T>, N> {
        CircStack::new(
            [const { MaybeUninit::uninit() }; N],
            0
        )
    }

    pub fn push(&mut self, value: T) {
        self.increment();
        self.buffer[self.tail] = value;
    }

    pub const fn pop_with_replacement(&mut self, replacement: T) -> T {
        let value = mem::replace(&mut self.buffer[self.tail], replacement);
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
        let CircStack { buffer, tail } = self;

        CircStack {
            buffer: buffer.map(MaybeUninit::new),
            tail
        }
    }

    // TODO: Verify this does what it should
    pub const fn rotate(&mut self, offset: isize) {
        if offset >= N as isize || offset <= -(N as isize) {
            panic!("TODO")
        }

        self.tail = (self.tail as isize + offset).rem_euclid(N as isize) as usize;
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
        CircStack::new(
            [item; N],
            0
        )
    }

    pub const fn pop_copy(&mut self) -> T {
        let value = self.buffer[self.tail];
        self.decrement();
        value
    }

    pub const fn last(&self) -> &T {
        &self.buffer[self.tail]
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
        let CircStack { buffer, tail } = self;

        Some(CircStack {
            buffer: buffer.transpose()?,
            tail,
        })
    }
}

impl<T: Default, const N: usize> Default for CircStack<T, N> {
    fn default() -> CircStack<T, N> {
        CircStack::new(
            [(); N].map(|_| T::default()),
            0
        )
    }
}

impl<T: Copy, const N: usize> TryFrom<&[T]> for CircStack<T, N> {
    type Error = TryFromSliceError;

    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        Ok(CircStack::new(
            <[T; N]>::try_from(value)?,
            0
        ))
    }
}

impl<T, const N: usize> From<[T; N]> for CircStack<T, N> {
    fn from(buffer: [T; N]) -> Self {
        CircStack::new(buffer, 0)
    }
}

// TODO: Could impl Index<isize>, but is it already getting too confusing?

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