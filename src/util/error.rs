use std::error::Error;
use std::fmt::{self, Display, Formatter};

use derive_more::{Display, Error, From, IsVariant, TryInto};

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

#[derive(Debug)]
pub struct CapacityOverflow;

impl Display for CapacityOverflow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Capacity overflow!")
    }
}

impl Error for CapacityOverflow {}

#[derive(Debug, Display, Error, From, TryInto, IsVariant)]
pub enum IndexOrCapOverflow {
    IndexOutOfBounds(IndexOutOfBounds),
    CapacityOverflow(CapacityOverflow),
}
