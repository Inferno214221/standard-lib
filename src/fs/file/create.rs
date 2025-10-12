use libc::{O_CREAT, O_EXCL, O_TMPFILE, O_TRUNC, c_int};

mod sealed {
    pub trait Sealed {
        const FLAGS: libc::c_int;
    }
}

use sealed::Sealed;

pub trait OpenMode: Sealed {}

pub trait Permanent: OpenMode {}

pub trait Temporary: OpenMode {}

pub enum NoCreate {}
impl Sealed for NoCreate {
    const FLAGS: c_int = 0;
}
impl OpenMode for NoCreate {}
impl Permanent for NoCreate {}

pub enum CreateIfMissing {}
impl Sealed for CreateIfMissing {
    const FLAGS: c_int = O_CREAT;
}
impl OpenMode for CreateIfMissing {}
impl Permanent for CreateIfMissing {}

pub enum CreateOrEmpty {}
impl Sealed for CreateOrEmpty {
    const FLAGS: c_int = O_CREAT | O_TRUNC;
}
impl OpenMode for CreateOrEmpty {}
impl Permanent for CreateOrEmpty {}

pub enum Create {}
impl Sealed for Create {
    const FLAGS: c_int = O_CREAT | O_EXCL;
}
impl OpenMode for Create {}
impl Permanent for Create {}

pub enum CreateTemp {}
impl Sealed for CreateTemp {
    const FLAGS: c_int = O_TMPFILE | O_EXCL;
}
impl OpenMode for CreateTemp {}
impl Temporary for CreateTemp {}

pub enum CreateUnlinked {}
impl Sealed for CreateUnlinked {
    const FLAGS: c_int = O_TMPFILE;
}
impl OpenMode for CreateUnlinked {}
impl Temporary for CreateUnlinked {}