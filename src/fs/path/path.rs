use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::ffi::{CString, OsStr, OsString};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::num::NonZero;
use std::ops::Deref;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use super::{DisplayPath, Rel};
use crate::collections::contiguous::Vector;
use crate::fs::path::{Ancestors, Components};
use crate::util::{self, sealed::Sealed};

use derive_more::IsVariant;

pub trait PathState: Sealed + Debug {}

#[derive(Clone)]
pub struct OwnedPath<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) inner: OsString,
}

#[repr(transparent)]
pub struct Path<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) inner: OsStr,
}

impl<T: AsRef<OsStr>, S: PathState> From<T> for OwnedPath<S> {
    fn from(value: T) -> Self {
        Self {
            _state: PhantomData,
            inner: sanitize_os_str(value.as_ref()),
        }
    }
}

impl<S: PathState> OwnedPath<S> {
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
        vec.reserve(other_path.len().get());
        vec.extend(other_path.inner.as_bytes().iter().cloned());
        let _ = mem::replace(
            &mut self.inner,
            OsString::from_vec(vec.into())
        );
    }

    pub fn pop(&mut self) -> Option<OwnedPath<Rel>> {
        // OsString doesn't make guarantees about its layout as a Vec, so we have to take and
        // convert it. Using mem::take replaces the value with an empty Vec, which avoids an
        // allocation. However, "" is not a valid path, so this function isn't currently unwind
        // safe. It would be dumb to perform an allocation so that OsString's internal Vec can be
        // taken and returned shortly after.
        // TODO: Make this unwind safe.
        let mut bytes = mem::take(&mut self.inner).into_vec();

        let mut index = bytes.len().checked_sub(1)?;

        while let Some(ch) = bytes.get(index) && *ch != b'/' {
            index = index.checked_sub(1)?;
        }

        let split = bytes.split_off(index);

        drop(mem::replace(&mut self.inner, OsString::from_vec(bytes)));

        Some(OwnedPath::<Rel> {
            _state: PhantomData,
            inner: OsString::from_vec(split),
        })
    }
}

impl<S: PathState> Path<S> {
    pub fn new<'a, O: AsRef<OsStr> + ?Sized>(value: &'a O) -> Cow<'a, Path<S>> {
        match validate_os_str(value.as_ref()) {
            Some(_) => Cow::Borrowed(unsafe { Path::from_unchecked(value) }),
            None    => Cow::Owned(OwnedPath::from(value)),
        }
    }

    pub fn from_checked<O: AsRef<OsStr> + ?Sized>(value: &O) -> Option<&Path<S>> {
        validate_os_str(value.as_ref())?;
        Some(unsafe { Path::from_unchecked(value) })
    }

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

    pub fn len(&self) -> NonZero<usize> {
        unsafe { NonZero::new(self.inner.len()).unwrap_unchecked() }
    }

    pub const fn as_os_str(&self) -> &OsStr {
        &self.inner
    }

    pub fn as_os_str_no_lead(&self) -> &OsStr {
        OsStr::from_bytes(&self.as_bytes()[1..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    // TODO: no_lead methods

    /// Returns the basename of this path (the OsStr following the last `/` in the path). This OsStr
    /// won't contain any instances of `/`.
    /// 
    /// See [`parent()`](Path::parent) for more info.
    pub fn basename(&self) -> &OsStr {
        let bytes = self.as_bytes();

        let mut index = bytes.len() - 1;

        while let Some(ch) = bytes.get(index) && *ch != b'/' {
            index -= 1;
        }

        OsStr::from_bytes(&bytes[(index + 1)..])
    }

    /// Returns the parent directory of this path (lexically speaking). The result is a Path with
    /// basename and the preceding slash removed, such that the following holds for any `path`.
    /// 
    /// ```
    /// # use standard_lib::fs::path::Path;
    /// let owned = OwnedPath::<Abs>::from("/my/path");
    /// let path: &Path<Abs> = &owned;
    /// let new_path = path.parent().join(Path::new(path.basename()));
    /// assert_eq!(path, new_path);
    /// ```
    /// 
    /// Because this method is the counterpart of [`basename`](Path::basename) and `basename` won't
    /// contain any `/`, the behavior when calling these methods on `"/"` is as follows:
    /// 
    /// ```
    /// # use standard_lib::fs::path::Path;
    /// assert_eq!(Path::root().basename(), "");
    /// assert_eq!(Path::root().parent(), Path::root());
    /// ```
    ///
    /// This behavior is also consistent with Unix defaults: the `..` entry in the root directory
    /// refers to the root itself.
    // TODO: Write doctest once a public constructor exists for OwnedPath and Path.
    pub fn parent(&self) -> &Self {
        let bytes = self.as_bytes();

        let mut index = bytes.len() - 1;

        while let Some(ch) = bytes.get(index) && *ch != b'/' {
            index -= 1;
        }
        
        // If we would return an empty string, instead include the first slash representing the
        // absolute or relative root.
        if index == 0 {
            index = 1;
        }

        unsafe { Path::from_unchecked(OsStr::from_bytes(&bytes[..index])) }
    }

    pub fn join<P: AsRef<Path<Rel>>>(&self, other: P) -> OwnedPath<S> {
        unsafe {
            OwnedPath::<S>::from_unchecked(
                [self.as_os_str(), other.as_ref().as_os_str()].into_iter().collect()
            )
        }
    }

    pub fn relative_to(&self, other: &Self) -> Option<&Path<Rel>> {
        // As a general note for path interpretation: paths on Linux have no encoding, with the only
        // constant being that they are delimited by b'/'. Because of this, we don't have to
        // consider encoding, and splitting by b"/" is always entirely valid because thats what
        // Linux does, even if b'/' is a later part of a variable-length character.
        match self.inner.as_bytes().strip_prefix(other.inner.as_bytes()) {
            None => None,
            // If there is no leading slash, strip_prefix matched only part of a component so
            // treat it as a fail.
            Some(replaced) if !replaced.starts_with(b"/") => None,
            // SAFETY: If the relative path starts with a b"/", then it is still a valid Path.
            Some(replaced) => unsafe {
                Some(Path::<Rel>::from_unchecked(OsStr::from_bytes(replaced)))
            },
        }
    }

    /// Creates an [`Iterator`] over the components of a `Path`. This iterator produces `Path<Rel>`s
    /// representing each `/`-separated string in the Path, from left to right.
    pub fn components<'a>(&'a self) -> Components<'a, S> {
        Components {
            _state: PhantomData,
            path: self.as_bytes(),
            head: 0,
        }
    }

    /// Creates an [`Iterator`] over the ancestors of a `Path`. This iterator produces `Path<S>`s
    /// representing each directory in the Path ordered with descending depth and ending with the
    /// Path itself.
    pub fn ancestors<'a>(&'a self) -> Ancestors<'a, S> {
        Ancestors {
            _state: PhantomData,
            path: self.as_bytes(),
            index: 0,
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

pub(crate) fn validate_os_str(value: &OsStr) -> Option<()> {
    let mut last_seq = Seq::Other;
    let mut bytes = value.as_bytes().iter();

    if bytes.next() != Some(&b'/') {
        None?
    }

    for ch in bytes {
        match (ch, last_seq) {
            (b'\0', _) => None?,
            (b'/', Seq::Slash) => None?,
            (b'/', Seq::SlashDot) => None?,
            (b'/', Seq::Other) => {
                last_seq = Seq::Slash;
            },
            (b'.', Seq::Slash) => {
                last_seq = Seq::SlashDot;
            },
            (_, Seq::Slash) => {
                last_seq = Seq::Other;
            },
            (_, Seq::SlashDot) => {
                last_seq = Seq::Other;
            },
            (_, Seq::Other) => (),
        }
    }

    if last_seq.is_slash() && value.len() > 1 {
        None?
    }

    Some(())
}

impl<S: PathState> From<OwnedPath<S>> for CString {
    fn from(value: OwnedPath<S>) -> Self {
        // SAFETY: OsString already guarantees that the internal string contains no '\0'.
        unsafe { CString::from_vec_unchecked(value.inner.into_vec()) }
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
        OwnedPath::<S> {
            _state: PhantomData,
            inner: self.as_os_str().to_owned(),
        }
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

impl<S: PathState> Debug for OwnedPath<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedPath")
            .field("<state>", &util::fmt::raw_type_name::<S>())
            .field("value", &self.inner)
            .finish()
    }
}

impl<S: PathState> Debug for Path<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Path")
            .field("<state>", &util::fmt::raw_type_name::<S>())
            .field("value", &&self.inner)
            .finish()
    }
}