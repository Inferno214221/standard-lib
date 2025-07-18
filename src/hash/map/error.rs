use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct IndexNoCap;

impl Display for IndexNoCap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to index HashMap with capacity 0!")
    }
}

impl Error for IndexNoCap {}
