use std::error::Error;
use std::io::RawOsError;

use derive_more::{Display, Error};

pub trait Panic: Error {
    fn panic(&self) -> ! {
        panic!("{}", self)
    }
}

#[derive(Debug, Display, Clone, Error)]
#[display("file descriptor corruption")]
pub struct BadFdPanic;
impl Panic for BadFdPanic {}

#[derive(Debug, Display, Clone, Error)]
#[display("invalid operation")]
pub struct InvalidOpPanic;
impl Panic for InvalidOpPanic {}

#[derive(Debug, Display, Clone, Error)]
#[display("invalid flag")]
pub struct InvalidFlagPanic;
impl Panic for InvalidFlagPanic {}

#[derive(Debug, Display, Clone, Error)]
#[display("pointer exceeded stack space")]
pub struct BadStackAddrPanic;
impl Panic for BadStackAddrPanic {}

#[derive(Debug, Display, Clone, Error)]
#[display("file descriptor doesn't refer to a directory")]
pub struct NotADirPanic;
impl Panic for NotADirPanic {}

#[derive(Debug, Display, Clone, Error)]
#[display("unexpected OS error with code: {_0}")]
pub struct UnexpectedErrorPanic(#[error(not(source))] pub RawOsError);
impl Panic for UnexpectedErrorPanic {}