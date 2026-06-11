use std::{ffi::OsStr, ops::{Add, AddAssign, Div, DivAssign}};

use crate::fs::{Abs, OwnedPath, Path, Rel, path::PathState};

// TODO: document for path-literal style syntax: a/"etc"/filename+".json"
#[allow(non_upper_case_globals)]
pub const a: &Path<Abs> = Path::ROOT;

// TODO: document for path-literal style syntax: r/".config"/filename+".json"
#[allow(non_upper_case_globals)]
pub const r: &Path<Rel> = Path::DOT;

impl<S: PathState, A: AsRef<Path<Rel>>> Div<A> for &Path<S> {
    type Output = OwnedPath<S>;

    fn div(self, rhs: A) -> Self::Output {
        self.join(rhs)
    }
}

impl<S: PathState, A: AsRef<Path<Rel>>> Div<A> for OwnedPath<S> {
    type Output = OwnedPath<S>;

    fn div(mut self, rhs: A) -> Self::Output {
        self.div_assign(rhs);
        self
    }
}

impl<S: PathState, A: AsRef<Path<Rel>>> DivAssign<A> for OwnedPath<S> {
    fn div_assign(&mut self, rhs: A) {
        self.push(rhs);
    }
}

impl<S: PathState, A: AsRef<OsStr>> Add<A> for &Path<S> {
    type Output = OwnedPath<S>;

    fn add(self, rhs: A) -> Self::Output {
        self.to_owned().add(rhs)
    }
}

impl<S: PathState, A: AsRef<OsStr>> Add<A> for OwnedPath<S> {
    type Output = OwnedPath<S>;

    fn add(mut self, rhs: A) -> Self::Output {
        self.add_assign(rhs);
        self
    }
}

impl<S: PathState, A: AsRef<OsStr>> AddAssign<A> for OwnedPath<S> {
    fn add_assign(&mut self, rhs: A) {
        let rhs = rhs.as_ref();
        validate_for_concat(rhs).expect("better error message");
        self.bytes.extend_from_slice(rhs.as_encoded_bytes());
    }
}

fn validate_for_concat(input: &OsStr) -> Option<()> {
    todo!()
}