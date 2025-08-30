use std::borrow::Borrow;
use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use super::{DisplayPath, Rel};
use crate::collections::contiguous::Vector;

use derive_more::IsVariant;

pub(crate) mod sealed {
    pub trait PathState {}
}

pub struct OwnedPath<State: sealed::PathState> {
    pub(crate) _phantom: PhantomData<fn() -> State>,
    pub(crate) inner: OsString,
}

#[repr(transparent)]
pub struct Path<State: sealed::PathState> {
    pub(crate) _phantom: PhantomData<fn() -> State>,
    pub(crate) inner: OsStr,
}

impl<S: sealed::PathState> OwnedPath<S> {
    pub(crate) fn from_os_str_sanitized(value: &OsStr) -> Self {
        Self {
            _phantom: PhantomData,
            inner: sanitize_os_str(value),
        }
    }

    pub const unsafe fn new_unchecked(inner: OsString) -> Self {
        Self {
            _phantom: PhantomData,
            inner,
        }
    }

    pub fn push<P: AsRef<Path<Rel>>>(&mut self, other: P) {
        let other_path = other.as_ref();
        let mut vec: Vector<u8> = mem::take(&mut self.inner).into_vec().into();
        vec.reserve(other_path.len());
        vec.extend(other_path.inner.as_bytes().iter().cloned());
        let _ = mem::replace(
            &mut self.inner,
            OsString::from_vec(vec.into())
        );
    }
}

impl<S: sealed::PathState> Path<S> {
    pub const unsafe fn new_unchecked(value: &OsStr) -> &Self {
        unsafe { &*(value as *const OsStr as *const Self) }
    }

    pub const unsafe fn new_unchecked_mut(value: &mut OsStr) -> &mut Self {
        unsafe { &mut *(value as *mut OsStr as *mut Self) }
    }

    pub const fn display<'a>(&'a self) -> DisplayPath<'a, S> {
        DisplayPath::<S> {
            _phantom: PhantomData,
            inner: self,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_os_str(&self) -> &OsStr {
        &self.inner
    }

    // pub fn as_bytes(&self) -> &[u8]

    // pub fn as_bytes_with_null(&self) -> &[u8]

    // pub fn as_ptr(&self) -> *const u8

    // pub fn basename(&self) -> &OsStr

    // pub fn parent(&self) -> Self

    pub fn join<P: AsRef<Path<Rel>>>(&self, other: P) -> OwnedPath<S> {
        unsafe {
            OwnedPath::<S>::new_unchecked(
                [self.as_os_str(), other.as_ref().as_os_str()].into_iter().collect()
            )
        }
    }

    pub fn relative(&self, other: &Self) -> Option<&Path<Rel>> {
        match self.inner.as_bytes().strip_prefix(other.inner.as_bytes()) {
            None => None,
            // If there is no leading slash, strip_prefix matched only part of a component so
            // treat it as a fail.
            Some(replaced) if !replaced.starts_with(b"/") => None,
            Some(replaced) => unsafe {
                Some(Path::<Rel>::new_unchecked(OsStr::from_bytes(replaced)))
            },
        }
    }
}

#[derive(Debug, Clone, Copy, IsVariant)]
enum Seq {
    Slash,
    SlashDot,
    Other,
}

pub(crate) fn sanitize_os_str(value: &OsStr) -> OsString {
    let mut last_seq = Seq::Other;
    let mut valid = Vector::with_cap(value.len() + 1);

    for ch in b"/".iter().chain(value.as_bytes().iter()).cloned() {
        match (ch, last_seq) {
            (b'\0', _) => (),
            (b'/', Seq::Slash) => (),
            (b'/', Seq::SlashDot) => {
                last_seq = Seq::Slash;
            },
            (b'/', Seq::Other) => {
                last_seq = Seq::Slash;
                valid.push(ch);
            },
            (b'.', Seq::Slash) => {
                last_seq = Seq::SlashDot;
            },
            (_, Seq::Slash) => {
                last_seq = Seq::Other;
                valid.push(ch);
            },
            (_, Seq::SlashDot) => {
                last_seq = Seq::Other;
                valid.push(b'.');
                valid.push(ch);
            },
            (_, Seq::Other) => {
                valid.push(ch);
            },
        }
    }

    if last_seq.is_slash() && valid.len() > 1 {
        valid.pop();
    }

    OsString::from_vec(valid.into())
}

impl<S: sealed::PathState> Deref for OwnedPath<S> {
    type Target = Path<S>;

    fn deref(&self) -> &Self::Target {
        unsafe { Path::<S>::new_unchecked(&self.inner) }
    }
}

impl<S: sealed::PathState> AsRef<Path<S>> for OwnedPath<S> {
    fn as_ref(&self) -> &Path<S> {
        self.deref()
    }
}

impl<S: sealed::PathState> Borrow<Path<S>> for OwnedPath<S> {
    fn borrow(&self) -> &Path<S> {
        self.as_ref()
    }
}

impl<S: sealed::PathState> AsRef<OsStr> for OwnedPath<S> {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_ref()
    }
}

impl<S: sealed::PathState> AsRef<OsStr> for Path<S> {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}
