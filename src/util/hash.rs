use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct BadHash<T: Eq>(T, usize);

impl<T: Eq> Hash for BadHash<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl<T: Eq> PartialEq for BadHash<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for BadHash<T> {}
