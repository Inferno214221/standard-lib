use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct IndexNoCap;

impl Display for IndexNoCap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to calculate indicies for hash-based collection with capacity 0!")
    }
}

impl Error for IndexNoCap {}
