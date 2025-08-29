use std::ffi::{OsStr, OsString};
use std::mem;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use derive_more::IsVariant;

use crate::collections::contiguous::Vector;
use crate::fs::path::RelPath;

pub(crate) mod sealed {
    use std::ffi::{OsStr, OsString};

    pub trait PathInternals {
        fn inner_mut(&mut self) -> &mut OsStr;
        fn inner(&self) -> &OsStr;
    }

    pub trait OwnedPathInternals {
        fn inner_mut(&mut self) -> &mut OsString;
        fn inner(&self) -> &OsString;
        unsafe fn new_unchecked(inner: OsString) -> Self;
    }
}

pub trait OwnedPathLike: sealed::OwnedPathInternals {
    fn push<P: AsRef<RelPath>>(&mut self, other: P) {
        let other_path = other.as_ref();
        let mut vec: Vector<u8> = mem::take(self.inner_mut()).into_vec().into();
        vec.reserve(other_path.len());
        vec.extend(other_path.inner.as_bytes().iter().cloned());
        let _ = mem::replace(
            self.inner_mut(),
            OsString::from_vec(vec.into())
        );
    }
}

pub trait PathLike: sealed::PathInternals {
    type Owned: OwnedPathLike + sealed::OwnedPathInternals;

    fn len(&self) -> usize {
        self.inner().len() - 1
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_os_str(&self) -> &OsStr {
        self.inner()
    }

    // fn as_bytes(&self) -> &[u8]

    // fn as_bytes_with_null(&self) -> &[u8]

    // fn as_ptr(&self) -> *const u8

    // fn basename(&self) -> &OsStr

    // fn parent(&self) -> Self

    fn join<P: AsRef<RelPath>>(&self, other: P) -> Self::Owned {
        use sealed::OwnedPathInternals;
        unsafe {
            Self::Owned::new_unchecked(
                [self.as_os_str(), other.as_ref().as_os_str()].into_iter().collect()
            )
        }
    }

    fn relative(&self, other: &Self) -> Option<&RelPath> {
        match self.inner().as_bytes().strip_prefix(other.inner().as_bytes()) {
            None => None,
            // If there is no leading slash, strip_prefix matched only part of a component so
            // treat it as a fail.
            Some(replaced) if !replaced.starts_with(b"/") => None,
            Some(replaced) => unsafe {
                Some(RelPath::new_unchecked(OsStr::from_bytes(replaced)))
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

pub(crate) fn sanitize_os_string(value: &OsStr, start: &[u8]) -> OsString {
    let mut last_seq = Seq::Other;
    let mut valid = Vector::with_cap(value.len() + 1);

    for ch in start.iter().chain(value.as_bytes().iter()).cloned() {
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
