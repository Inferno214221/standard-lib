use std::ffi::OsStr;
use std::marker::PhantomData;
use std::os::unix::ffi::OsStrExt;

use crate::fs::path::{Path, PathState, Rel};

// TODO: impl other Iterator traits

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
            Some(Path::from_unchecked(OsStr::from_bytes(res)))
        }
    }
}

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
            Some(Path::from_unchecked(OsStr::from_bytes(
                &self.path[..self.index]
            )))
        }
    }
}