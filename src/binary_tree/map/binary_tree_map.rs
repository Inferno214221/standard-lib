use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};

use super::{Branch, Iter};

pub struct BinaryTreeMap<K: Ord, V> {
    pub(crate) root: Branch<K, V>,
    pub(crate) len: usize,
}

impl<K: Ord, V> BinaryTreeMap<K, V> {
    pub const fn new() -> BinaryTreeMap<K, V> {
        BinaryTreeMap {
            root: Branch(None),
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.len += 1;
        self.root.insert(key, value)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        let result = self.root.remove_entry(key);
        if result.is_some() {
            self.len -= 1;
        }
        result
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        self.remove_entry(key).map(|e| e.1)
    }

    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        self.root.get_entry(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        self.get_entry(key).map(|e| e.1)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        self.root.get_mut(key)
    }

    pub fn contains<Q>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized
    {
        self.root.contains(key)
    }

    pub fn first_entry(&self) -> Option<(&K, &V)> {
        self.root.first_entry()
    }

    pub fn first(&self) -> Option<&V> {
        self.first_entry().map(|e| e.1)
    }

    pub fn take_first_entry(&mut self) -> Option<(K, V)> {
        self.root.take_first_entry()
    }

    pub fn take_first(&mut self) -> Option<V> {
        self.take_first_entry().map(|e| e.1)
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }
}

impl<K: Ord, V> Default for BinaryTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Debug, V: Debug> Debug for BinaryTreeMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryTreeMap")
            .field_with("nodes", |f| write!(f, "\n{:?}\n", &self.root))
            .field("len", &self.len)
            .finish()
    }
}

impl<K: Ord + Debug, V: Debug> Display for BinaryTreeMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}