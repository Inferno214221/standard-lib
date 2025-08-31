#![warn(missing_docs)]

pub mod alloc;
pub mod error;
pub mod fmt;
pub mod hash;
pub mod panic;
pub mod result;

pub(crate) mod sealed {
    pub trait Sealed {}
}
