use std::hash::{BuildHasher, Hash, Hasher};

#[derive(Debug)]
#[allow(unused)]
pub struct ManualHash<T: Eq> {
    hash: u64,
    value: T,
}

impl<T: Eq> ManualHash<T> {
    #[allow(unused)]
    pub const fn new(hash: u64, value: T) -> ManualHash<T> {
        ManualHash {
            hash,
            value,
        }
    }

    #[allow(unused)]
    pub fn value(self) -> T {
        self.value
    }
}

impl<T: Eq> Hash for ManualHash<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: Eq> PartialEq for ManualHash<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for ManualHash<T> {}

#[derive(Debug)]
pub struct BadHasher {
    state: u64,
}

impl Hasher for BadHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut offset = 0_u64;
        for byte in bytes {
            self.state ^= (*byte as u64) << (offset * 8);
            offset = (offset + 1) % 8;
        }
    }
}

#[derive(Debug, Default)]
pub struct BadHasherBuilder;

impl BuildHasher for BadHasherBuilder {
    type Hasher = BadHasher;

    fn build_hasher(&self) -> Self::Hasher {
        BadHasher {
            state: 0
        }
    }
}
