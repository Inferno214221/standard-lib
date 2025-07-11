use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct IndexOutOfBounds {
    pub index: usize,
    pub len: usize,
}

impl Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "index {} out of bounds for collection with {} elements", self.index, self.len)
    }
}

impl Error for IndexOutOfBounds {}
