use std::fmt::Debug;

use libc::{O_RDONLY, O_RDWR, O_WRONLY, c_int};

use crate::util::sealed::Sealed;

pub trait AccessMode: Sealed + Debug {
    const FLAGS: libc::c_int;
}

pub trait Read: AccessMode {}

pub trait Write: AccessMode {}

#[derive(Debug)]
pub enum ReadOnly {}

impl Sealed for ReadOnly {}

impl AccessMode for ReadOnly {
    const FLAGS: c_int = O_RDONLY;
}

impl Read for ReadOnly {}

#[derive(Debug)]
pub enum WriteOnly {}

impl Sealed for WriteOnly {}

impl AccessMode for WriteOnly {
    const FLAGS: c_int = O_WRONLY;
}

impl Write for ReadOnly {}

#[derive(Debug)]
pub enum ReadWrite {}

impl Sealed for ReadWrite {}

impl AccessMode for ReadWrite {
    const FLAGS: c_int = O_RDWR;
}

impl Read for ReadWrite {}

impl Write for ReadWrite {}