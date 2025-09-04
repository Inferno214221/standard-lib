use std::error::Error;
use std::io::RawOsError;

use derive_more::{Display, Error};

pub trait Panic: Error {
    fn panic(&self) -> ! {
        panic!("{}", self)
    }
}

#[derive(Debug, Display, Error)]
#[display("file descriptor corruption")]
pub struct BadFdPanic;
impl Panic for BadFdPanic {}

#[derive(Debug, Display, Error)]
#[display("invalid operation")]
pub struct InvalidOpPanic;
impl Panic for InvalidOpPanic {}

#[derive(Debug, Display, Error)]
#[display("pointer exceeded stack space")]
pub struct BadStackAddrPanic;
impl Panic for BadStackAddrPanic {}

#[derive(Debug, Display, Error)]
#[display("unexpected OS error with code: {_0}")]
pub struct UnexpectedErrorPanic(#[error(not(source))] pub RawOsError);
impl Panic for UnexpectedErrorPanic {}