use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ffi::{CString, OsStr, OsString};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use super::{DisplayPath, Rel};
use crate::collections::contiguous::Vector;
use crate::util::sealed::Sealed;

use derive_more::IsVariant;

pub trait PathState: Sealed + Debug {}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct OwnedPath<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) inner: OsString,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Path<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) inner: OsStr,
}

impl<S: PathState> OwnedPath<S> {
    pub(crate) fn from_os_str_sanitized(value: &OsStr) -> Self {
        Self {
            _state: PhantomData,
            inner: sanitize_os_str(value),
        }
    }

    pub const unsafe fn from_unchecked(inner: OsString) -> Self {
        Self {
            _state: PhantomData,
            inner,
        }
    }

    pub fn as_path(&self) -> &Path<S> {
        self
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

impl<S: PathState> Path<S> {
    pub unsafe fn from_unchecked<O: AsRef<OsStr> + ?Sized>(value: &O) -> &Self {
        // SAFETY: Path<S> is `repr(transparent)`, so to it has the same layout as OsStr.
        unsafe { &*(value.as_ref() as *const OsStr as *const Self) }
    }

    pub unsafe fn from_unchecked_mut<O: AsMut<OsStr> + ?Sized>(value: &mut O) -> &mut Self {
        // SAFETY: Path<S> is `repr(transparent)`, so to it has the same layout as OsStr.
        unsafe { &mut *(value.as_mut() as *mut OsStr as *mut Self) }
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

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    // pub fn basename(&self) -> &OsStr

    // pub fn parent(&self) -> Self

    pub fn join<P: AsRef<Path<Rel>>>(&self, other: P) -> OwnedPath<S> {
        unsafe {
            OwnedPath::<S>::from_unchecked(
                [self.as_os_str(), other.as_ref().as_os_str()].into_iter().collect()
            )
        }
    }

    pub fn relative(&self, other: &Self) -> Option<&Path<Rel>> {
        match self.inner.as_bytes().strip_prefix(other.inner.as_bytes()) {
            None => None,
            // If there is no leading slash, strip_prefix matched only part of a component so
            // treat it as a fail.
            // FIXME: It is possible that b"/" is the second part of a multi-byte character.
            Some(replaced) if !replaced.starts_with(b"/") => None,
            // SAFETY: If the relative path starts with a b"/", then it is still a valid Path.
            Some(replaced) => unsafe {
                Some(Path::<Rel>::from_unchecked(OsStr::from_bytes(replaced)))
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

// Unfortunately, it's cheaper to copy all values one by one that constantly move all bytes back and
// forward with insertions and removals.
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

impl<S: PathState> From<OwnedPath<S>> for CString {
    fn from(value: OwnedPath<S>) -> Self {
        let mut bytes = value.inner.into_vec();
        bytes.push(b'\0');
        // SAFETY: OsString already guarantees that the internal string contains no '\0'.
        unsafe { CString::from_vec_with_nul_unchecked(bytes) }
    }
}

impl<S: PathState> Deref for OwnedPath<S> {
    type Target = Path<S>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: OwnedPath upholds the same invariants as Path.
        unsafe { Path::<S>::from_unchecked(&self.inner) }
    }
}

impl<S: PathState> AsRef<Path<S>> for OwnedPath<S> {
    fn as_ref(&self) -> &Path<S> {
        self.deref()
    }
}

// Apparently there isn't a blanket impl for this?
impl<S: PathState> AsRef<Path<S>> for Path<S> {
    fn as_ref(&self) -> &Path<S> {
        self
    }
}

impl<S: PathState> Borrow<Path<S>> for OwnedPath<S> {
    fn borrow(&self) -> &Path<S> {
        self.as_ref()
    }
}

impl<S: PathState> AsRef<OsStr> for Path<S> {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}

impl<S: PathState> ToOwned for Path<S> {
    type Owned = OwnedPath<S>;

    fn to_owned(&self) -> Self::Owned {
        OwnedPath::<S>::from_os_str_sanitized(self.as_os_str())
    }
}

impl<S: PathState> PartialEq for OwnedPath<S> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().inner == other.as_ref().inner
    }
}

impl<S: PathState> PartialEq for Path<S> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<S: PathState> PartialEq<Path<S>> for OwnedPath<S> {
    fn eq(&self, other: &Path<S>) -> bool {
        self.as_ref().inner == other.inner
    }
}

impl<S: PathState> PartialEq<OwnedPath<S>> for Path<S> {
    fn eq(&self, other: &OwnedPath<S>) -> bool {
        self.inner == other.as_ref().inner
    }
}

impl<S: PathState> Eq for OwnedPath<S> {}

impl<S: PathState> Eq for Path<S> {}

impl<S: PathState> PartialOrd for OwnedPath<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: PathState> Ord for OwnedPath<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().inner.cmp(&other.as_ref().inner)
    }
}

impl<S: PathState> PartialOrd for Path<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: PathState> Ord for Path<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<S: PathState> PartialOrd<Path<S>> for OwnedPath<S> {
    fn partial_cmp(&self, other: &Path<S>) -> Option<Ordering> {
        Some(self.as_ref().cmp(other))
    }
}

impl<S: PathState> PartialOrd<OwnedPath<S>> for Path<S> {
    fn partial_cmp(&self, other: &OwnedPath<S>) -> Option<Ordering> {
        Some(self.cmp(other.as_ref()))
    }
}
