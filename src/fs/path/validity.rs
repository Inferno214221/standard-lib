use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use derive_more::IsVariant;

use crate::collections::contiguous::Vector;

#[derive(Debug, Clone, Copy, IsVariant)]
enum Seq {
    Slash,
    SlashDot,
    Other,
}

// Unfortunately, I think it's cheaper to copy all values one by one that constantly move all bytes
// back and forward with insertions and removals. O(n) as opposed to O(n^2).
pub fn sanitize(value: &OsStr) -> OsString {
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

    if valid.len() > 1 {
        match last_seq {
            Seq::Slash    => valid.pop(),
            Seq::SlashDot => valid.pop(),
            Seq::Other    => None,
        };
    }

    OsString::from_vec(valid.into())
}

pub fn validate(value: &OsStr) -> Option<()> {
    let mut bytes = value.as_bytes().iter();

    if bytes.next() != Some(&b'/') {
        None?
    }
    let mut last_seq = Seq::Slash;

    for ch in bytes {
        last_seq = match (ch, last_seq) {
            (b'\0', _)            => None?,
            (b'/', Seq::Slash)    => None?,
            (b'/', Seq::SlashDot) => None?,
            (b'/', Seq::Other)    => Seq::Slash,
            (b'.', Seq::Slash)    => Seq::SlashDot,
            (_, Seq::Slash)       => Seq::Other,
            (_, Seq::SlashDot)    => Seq::Other,
            (_, Seq::Other)       => Seq::Other,
        };
    }

    match last_seq {
        Seq::Slash if value.len() > 1 => None,
        Seq::SlashDot                 => None,
        _                             => Some(()),
    }
}