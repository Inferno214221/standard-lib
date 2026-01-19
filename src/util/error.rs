use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct IndexOutOfBounds {
    pub index: usize,
    pub len: usize,
}

impl Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Index {} out of bounds for collection with {} elements!", self.index, self.len)
    }
}

impl Error for IndexOutOfBounds {}

/// TODO
///
/// Note: This error is no longer returned by any public part of the API, but it is thrown during
/// panics. This is because a capacity overflow has such a small chance of occurring that it isn't
/// worth handling in most placed. Most machines wouldn't have enough memory to overflow a non-ZST
/// collection with a u64 length.
#[derive(Debug)]
pub struct CapacityOverflow;

impl Display for CapacityOverflow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Capacity overflow!")
    }
}

impl Error for CapacityOverflow {}

#[derive(Debug)]
pub struct NoValueForKey;

impl Display for NoValueForKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "No values is associated with the provided key.")
    }
}

impl Error for NoValueForKey {}
