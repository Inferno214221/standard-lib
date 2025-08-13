use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::slice;

use derive_more::IsVariant;

use crate::collections::contiguous::Vector;
use crate::fs::path::RelPath;

pub(crate) mod sealed {
    use std::ffi::OsString;

    pub trait PathInternals {
        fn inner_mut(&mut self) -> &mut OsString;
        fn inner(&self) -> &OsString;
    }
}

pub trait PathLike: sealed::PathInternals {    
    fn len(&self) -> usize {
        self.inner().len() - 1
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_os_str(&self) -> &OsStr {
        self.inner().as_os_str()
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    fn as_bytes_with_null(&self) -> &[u8] {
        self.inner().as_encoded_bytes()
    }

    fn as_ptr(&self) -> *const u8 {
        self.inner().as_encoded_bytes().as_ptr()
    }

    fn join(&mut self, other: &RelPath) {
        let mut vec: Vector<u8> = mem::take(self.inner_mut()).into_encoded_bytes().into();
        vec.pop();
        vec.reserve(other.len());
        vec.extend(other.inner.as_bytes().iter().skip(1).cloned());
        let _ = mem::replace(
            self.inner_mut(),
            unsafe { OsString::from_encoded_bytes_unchecked(vec.into()) }
        );
    }

    // fn basename(&self) -> OsString

    // fn parent(&self) -> Self

    // fn metadata(&self) -> Result<Metadata>;

    // fn open(&self) -> Union(File, Dir, etc.) Union should hold metadata too?

    // no follow with O_NOFOLLOW
    // to open as a specific type, use File::open or Dir::open

    // fn canonicalize

    // fn exists/try_exists

    // fn read_shortcuts

    // NOTE: Symlinks can't be opened, so all symlink-related APIs need to be handled here.

    // fn is_symlink

    // fn symlink_metadata

    // fn read_link

    // type agnostic methods, e.g. copy, move, rename, etc. chown, chmod?
}

#[derive(Debug, Clone, Copy, IsVariant)]
enum Seq {
    Slash,
    SlashDot,
    Other
}

pub(crate) fn sanitize_os_string(value: &OsStr, start: &[u8]) -> OsString {
    let mut last_seq = Seq::Other;
    let mut valid = Vector::with_cap(value.len() + 1);

    for ch in start.iter().chain(value.as_encoded_bytes().iter()).cloned() {
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

    if last_seq.is_slash() {
        valid.pop();
    }
    valid.push(b'\0');
    
    unsafe { OsString::from_encoded_bytes_unchecked(valid.into()) }
}