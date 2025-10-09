use libc::{O_CREAT, O_EXCL, O_TMPFILE, O_TRUNC, c_int};

use crate::util::sealed::Sealed;

pub trait OpenMode: Sealed {
    const FLAGS: c_int;
}

pub trait Permanent: OpenMode {}

pub trait Temporary: OpenMode {}

pub enum NoCreate {}
impl Sealed for NoCreate {}
impl OpenMode for NoCreate {
    const FLAGS: c_int = 0;
}
impl Permanent for NoCreate {}

pub enum CreateIfMissing {}
impl Sealed for CreateIfMissing {}
impl OpenMode for CreateIfMissing {
    const FLAGS: c_int = O_CREAT;
}
impl Permanent for CreateIfMissing {}

pub enum CreateOrEmpty {}
impl Sealed for CreateOrEmpty {}
impl OpenMode for CreateOrEmpty {
    const FLAGS: c_int = O_CREAT | O_TRUNC;
}
impl Permanent for CreateOrEmpty {}

pub enum Create {}
impl Sealed for Create {}
impl OpenMode for Create {
    const FLAGS: c_int = O_CREAT | O_EXCL;
}
impl Permanent for Create {}

pub enum CreateTemp {}
impl Sealed for CreateTemp {}
impl OpenMode for CreateTemp {
    const FLAGS: c_int = O_TMPFILE | O_EXCL;
}
impl Permanent for CreateTemp {}

pub enum CreateUnlinked {}
impl Sealed for CreateUnlinked {}
impl OpenMode for CreateUnlinked {
    const FLAGS: c_int = O_TMPFILE;
}
impl Permanent for CreateUnlinked {}