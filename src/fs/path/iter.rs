use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::mem;

use crate::fs::OwnedPath;
use crate::fs::path::{Path, PathState, Rel};

// TODO: OwnedComponents?
pub struct IntoComponents<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) path: Vec<u8>,
}

impl<S: PathState> Iterator for IntoComponents<S> {
    type Item = OwnedPath<Rel>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            None?
        }
        let mut tail = 1;

        while let Some(ch) = self.path.get(tail) && *ch != b'/' {
            tail += 1;
        }

        let remainder = self.path.split_off(tail);

        Some(unsafe { OwnedPath::from_unchecked_bytes(
            mem::replace(&mut self.path, remainder)
        ) })
    }
}

impl<S: PathState> FusedIterator for IntoComponents<S> {}

pub struct Components<'a, State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) path: &'a [u8],
    pub(crate) head: usize,
}

impl<'a, S: PathState> Iterator for Components<'a, S> {
    type Item = &'a Path<Rel>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head >= self.path.len() {
            None?
        }
        let mut tail = self.head + 1;
        
        while let Some(ch) = self.path.get(tail) && *ch != b'/' {
            tail += 1;
        }

        let res = &self.path[self.head..tail];
        self.head = tail;

        unsafe {
            Some(Path::from_unchecked_bytes(res))
        }
    }
}

impl<'a, S: PathState> FusedIterator for Components<'a, S> {}

pub struct Ancestors<'a, State: PathState + 'a> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) path: &'a [u8],
    pub(crate) index: usize,
}

impl<'a, S: PathState> Iterator for Ancestors<'a, S> {
    type Item = &'a Path<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.path.len() {
            None?
        }
        self.index += 1;

        while let Some(ch) = self.path.get(self.index) && *ch != b'/' {
            self.index += 1;
        }

        unsafe {
            Some(Path::from_unchecked_bytes(
                &self.path[..self.index]
            ))
        }
    }
}

impl<'a, S: PathState> FusedIterator for Ancestors<'a, S> {}