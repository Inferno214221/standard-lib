use libc::{O_CREAT, O_EXCL, O_TRUNC, c_int};

use crate::util::sealed::Sealed;

pub trait OpenMode: Sealed {
    const FLAGS: c_int;
}

pub enum NoCreate {}

impl Sealed for NoCreate {}

impl OpenMode for NoCreate {
    const FLAGS: c_int = 0;
}

pub enum CreateIfMissing {}

impl Sealed for CreateIfMissing {}

impl OpenMode for CreateIfMissing {
    const FLAGS: c_int = O_CREAT;
}

pub enum CreateOrEmpty {}

impl Sealed for CreateOrEmpty {}

impl OpenMode for CreateOrEmpty {
    const FLAGS: c_int = O_CREAT | O_TRUNC;
}

pub enum Create {}

impl Sealed for Create {}

impl OpenMode for Create {
    const FLAGS: c_int = O_CREAT | O_EXCL;
}