use libc::{O_RDONLY, O_RDWR, O_WRONLY, c_int};

mod sealed {
    pub trait Sealed {
        const FLAGS: libc::c_int;
    }
}

use sealed::Sealed;

pub trait AccessMode: Sealed {}

pub trait Read: AccessMode {}

pub trait Write: AccessMode {}

pub enum ReadOnly {}
impl Sealed for ReadOnly {
    const FLAGS: c_int = O_RDONLY;
}
impl AccessMode for ReadOnly {}
impl Read for ReadOnly {}

pub enum WriteOnly {}
impl Sealed for WriteOnly {
    const FLAGS: c_int = O_WRONLY;
}
impl AccessMode for WriteOnly {}
impl Write for WriteOnly {}

pub enum ReadWrite {}
impl Sealed for ReadWrite {
    const FLAGS: c_int = O_RDWR;
}
impl AccessMode for ReadWrite {}
impl Read for ReadWrite {}
impl Write for ReadWrite {}