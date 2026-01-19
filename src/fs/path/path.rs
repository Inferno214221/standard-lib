use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::ffi::{CString, OsStr, OsString};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::Deref;
use std::os::unix::ffi::OsStrExt;

use super::{DisplayPath, Rel};
use crate::fs::path::{Ancestors, Components, validity};
use crate::util::error::CapacityOverflow;
use crate::util::result::ResultExtension;
use crate::util::{self, sealed::Sealed};

pub trait PathState: Sealed + Debug {}

pub struct OwnedPath<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) bytes: Vec<u8>,
}

#[repr(transparent)]
pub struct Path<State: PathState> {
    pub(crate) _state: PhantomData<fn() -> State>,
    pub(crate) bytes: [u8],
}

impl<T: AsRef<OsStr>, S: PathState> From<T> for OwnedPath<S> {
    fn from(value: T) -> Self {
        Self {
            _state: PhantomData,
            bytes: validity::sanitize(value.as_ref()),
        }
    }
}

impl<S: PathState> OwnedPath<S> {
    pub unsafe fn from_unchecked<O: Into<OsString>>(inner: O) -> Self {
        Self {
            _state: PhantomData,
            bytes: inner.into().into_encoded_bytes(),
        }
    }

    pub unsafe fn from_unchecked_bytes(inner: Vec<u8>) -> Self {
        Self {
            _state: PhantomData,
            bytes: inner,
        }
    }

    pub fn as_path(&self) -> &Path<S> {
        self
    }

    pub fn push<P: Into<OwnedPath<Rel>>>(&mut self, other: P) {
        let other_path = other.into();

        match self.len().get().checked_add(other_path.len().get()) {
            // TODO: Standardize crate panic method.
            Some(l) if l > isize::MAX as usize => Err(CapacityOverflow).throw(),
            None                               => Err(CapacityOverflow).throw(),
            Some(_)                            => (),
        }

        // We've already determined that this won't surpass size::MAX.
        self.bytes.reserve(other_path.len().get());

        // Path is designed in such a way that two valid Paths can't be concatenated to create an
        // invalid Path.
        self.bytes.extend(other_path.bytes);
    }

    pub fn pop(&mut self) -> Option<OwnedPath<Rel>> {
        if self.bytes.len() == 1 {
            // If a Path has length 1, it only contains b'/', and nothing can be popped from it.
            return None;
        }

        // A Path has at least one character, so subtracting 1 from the length can't be less than 0.
        let mut index = self.bytes.len() - 1;

        while let Some(ch) = self.bytes.get(index) && *ch != b'/' {
            // A Path has to start with a b'/', so entering this loop already confirms that there is
            // another character preceding this one.
            index -= 1;
        }

        // The index is guaranteed to be less than the length of bytes, so this can't panic.
        let split = self.bytes.split_off(index);

        if self.bytes.is_empty() {
            // We've literally just checked that bytes is empty, to a single push can't panic.
            self.bytes.push(b'/');
        }

        Some(OwnedPath::<Rel> {
            _state: PhantomData,
            bytes: split,
        })
    }
}

impl<S: PathState> Path<S> {
    pub fn new<'a, O: AsRef<OsStr> + ?Sized>(value: &'a O) -> Cow<'a, Path<S>> {
        match validity::validate(value.as_ref()) {
            Some(_) => Cow::Borrowed(unsafe { Path::from_unchecked(value) }),
            None    => Cow::Owned(OwnedPath::from(value)),
        }
    }

    pub fn from_checked<O: AsRef<OsStr> + ?Sized>(value: &O) -> Option<&Path<S>> {
        validity::validate(value.as_ref())?;
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

    pub unsafe fn from_unchecked_bytes(value: &[u8]) -> &Self {
        // SAFETY: Path<S> is `repr(transparent)`, so to it has the same layout as &[u8].
        unsafe { &*(value.as_ref() as *const [u8] as *const Self) }
    }

    pub const fn display<'a>(&'a self) -> DisplayPath<'a, S> {
        DisplayPath::<S> {
            _phantom: PhantomData,
            inner: self,
        }
    }

    pub fn len(&self) -> NonZero<usize> {
        unsafe { NonZero::new(self.bytes.len()).unwrap_unchecked() }
    }

    pub fn as_os_str(&self) -> &OsStr {
        OsStr::from_bytes(&self.bytes)
    }

    pub fn as_os_str_no_lead(&self) -> &OsStr {
        OsStr::from_bytes(&self.as_bytes()[1..])
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
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
        let mut bytes = Vec::with_capacity(self.bytes.len() + other.as_ref().bytes.len());
        bytes.extend(&self.bytes);
        bytes.extend(&other.as_ref().bytes);
        unsafe {
            OwnedPath::<S>::from_unchecked_bytes(bytes)
        }
    }

    pub fn relative_to(&self, other: &Self) -> Option<&Path<Rel>> {
        // As a general note for path interpretation: paths on Linux have no encoding, with the only
        // constant being that they are delimited by b'/'. Because of this, we don't have to
        // consider encoding, and splitting by b"/" is always entirely valid because thats what
        // Linux does, even if b'/' is a later part of a variable-length character.
        match self.bytes.strip_prefix(&other.bytes) {
            None => None,
            // If there is no leading slash, strip_prefix matched only part of a component so
            // treat it as a fail.
            Some(replaced) if !replaced.starts_with(b"/") => None,
            // SAFETY: If the relative path starts with a b"/", then it is still a valid Path.
            Some(replaced) => unsafe {
                Some(Path::<Rel>::from_unchecked_bytes(replaced))
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

impl<S: PathState> From<OwnedPath<S>> for CString {
    fn from(value: OwnedPath<S>) -> Self {
        // SAFETY: OsString already guarantees that the internal string contains no '\0'.
        unsafe { CString::from_vec_unchecked(value.bytes) }
    }
}

impl<S: PathState> Deref for OwnedPath<S> {
    type Target = Path<S>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: OwnedPath upholds the same invariants as Path.
        unsafe { Path::<S>::from_unchecked(OsStr::from_bytes(&self.bytes)) }
    }
}

impl<S: PathState> From<&Path<S>> for OwnedPath<S> {
    fn from(value: &Path<S>) -> Self {
        value.to_owned()
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

// AsRef<OsStr> causes conflicting implementations and makes it slightly too easy to interpret a
// Path as an OsStr. The same functionality has been moved to Path::as_os_str(), which requires
// explicit usage. Otherwise, users can accidentally convert between Path<Abs> and Path<Rel>.
// impl<S: PathState> AsRef<OsStr> for Path<S> {
//     fn as_ref(&self) -> &OsStr {
//         &self.inner
//     }
// }

impl<S: PathState> ToOwned for Path<S> {
    type Owned = OwnedPath<S>;

    fn to_owned(&self) -> Self::Owned {
        OwnedPath::<S> {
            _state: PhantomData,
            bytes: self.bytes.to_vec(),
        }
    }
}

impl<S: PathState> Clone for OwnedPath<S> {
    fn clone(&self) -> Self {
        Self {
            _state: PhantomData,
            bytes: self.bytes.clone()
        }
    }
}

impl<S: PathState> PartialEq for OwnedPath<S> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().bytes == other.as_ref().bytes
    }
}

impl<S: PathState> PartialEq for Path<S> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<S: PathState> PartialEq<Path<S>> for OwnedPath<S> {
    fn eq(&self, other: &Path<S>) -> bool {
        self.as_ref().bytes == other.bytes
    }
}

impl<S: PathState> PartialEq<OwnedPath<S>> for Path<S> {
    fn eq(&self, other: &OwnedPath<S>) -> bool {
        self.bytes == other.as_ref().bytes
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
        self.as_ref().bytes.cmp(&other.as_ref().bytes)
    }
}

impl<S: PathState> PartialOrd for Path<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: PathState> Ord for Path<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
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
            .field("value", &OsStr::from_bytes(&self.bytes))
            .finish()
    }
}

impl<S: PathState> Debug for Path<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Path")
            .field("<state>", &util::fmt::raw_type_name::<S>())
            .field("value", &OsStr::from_bytes(&self.bytes))
            .finish()
    }
}