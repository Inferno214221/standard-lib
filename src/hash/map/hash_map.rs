use std::borrow::Borrow;
use std::cmp;
use std::hash::{BuildHasher, Hash, RandomState};
use std::mem;

use super::{IterMut, Iter, IntoKeys, Keys, IntoValues, ValuesMut, Values};
use crate::contiguous::Array;

const MIN_ALLOCATED_CAP: usize = 2;

const GROWTH_FACTOR: usize = 2;

const LOAD_FACTOR_NUMERATOR: usize = 4;
const LOAD_FACTOR_DENOMINATOR: usize = 5;

pub struct HashMap<K: Hash + Eq, V, B: BuildHasher = RandomState> {
    pub(crate) arr: Array<Bucket<K, V>>,
    pub(crate) len: usize,
    pub(crate) hasher: B,
}

pub(crate) type Bucket<K, V> = Option<Box<(K, V)>>;

impl<K: Hash + Eq, V, B: BuildHasher + Default> HashMap<K, V, B> {
    pub fn new() -> HashMap<K, V, B> {
        HashMap {
            arr: Array::new(),
            len: 0,
            hasher: B::default()
        }
    }

    pub fn with_cap(cap: usize) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::repeat_default(cap),
            len: 0,
            hasher: B::default()
        }
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    pub fn with_hasher(hasher: B) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::new(),
            len: 0,
            hasher,
        }
    }

    pub fn with_cap_and_hasher(cap: usize, hasher: B) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::repeat_default(cap),
            len: 0,
            hasher,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn cap(&self) -> usize {
        self.arr.size
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.should_grow() {
            self.grow()
        }
        
        let index = self.find_index_for_key(&key);

        // The Bucket at index is either empty or contains an equal key.
        match &mut self.arr[index] {
            Some(existing) => {
                // Replace the value with the provided one.
                Some(mem::replace(
                    &mut existing.1,
                    value
                ))
            },
            None => {
                // Create a new Bucket with the provided values.
                self.arr[index] = Some(Box::new((key, value)));
                self.len += 1;
                None
            },
        }
    }

    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        // We're introducing a new type parameter here, Q which represents a borrowed version of K
        // where equality and hashing carries over the borrow.
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let index = self.find_index_for_borrowed(key);

        // If the Bucket at index is empty, the map doesn't contain the key.
        match &self.arr[index] {
            Some(existing) => Some((&existing.0, &existing.1)),
            None => None,
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        // We're introducing a new type parameter here, Q which represents a borrowed version of K
        // where equality and hashing carries over the borrow.
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let index = self.find_index_for_borrowed(key);

        // If the Bucket at index is empty, the map doesn't contain the key.
        match &self.arr[index] {
            Some(existing) => Some(&existing.1),
            None => None,
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let index = self.find_index_for_borrowed(key);

        // If the Bucket at index is empty, the map doesn't contain the key.
        match &mut self.arr[index] {
            Some(existing) => Some(&mut existing.1),
            None => None,
        }
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let index = self.find_index_for_borrowed(key);

        // If the Bucket at index is empty, the map doesn't contain the key.
        match mem::take(&mut self.arr[index]) {
            Some(entry) => {
                self.len -= 1;
                Some(*entry)
            },
            None => None,
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.remove_entry(key).map(|e| e.1)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let index = self.find_index_for_borrowed(key);

        self.arr[index].is_some()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys(self.into_iter())
    }

    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys(self.iter())
    }
    
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    pub fn values_mut<'a>(&'a mut self) -> ValuesMut<'a, K, V> {
        ValuesMut(self.iter_mut())
    }

    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values(self.iter())
    }

    pub fn reserve(&mut self, extra: usize) {
        let new_cap = self.len.strict_add(extra * LOAD_FACTOR_DENOMINATOR / LOAD_FACTOR_NUMERATOR);

        self.realloc_with_cap(new_cap);
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    pub(crate) const fn should_grow(&self) -> bool {
        self.len > self.arr.size * LOAD_FACTOR_NUMERATOR / LOAD_FACTOR_DENOMINATOR
    }

    pub(crate) fn grow(&mut self) {
        let new_cap = cmp::max(self.cap() * GROWTH_FACTOR, MIN_ALLOCATED_CAP);

        self.realloc_with_cap(new_cap)
    }

    pub(crate) fn realloc_with_cap(&mut self, new_cap: usize) {
        // Replace the Array first so that we can consume the old Array.
        let old_arr = mem::replace(&mut self.arr, Array::repeat_default(new_cap));

        for entry in old_arr.into_iter().flatten() {
            let index = self.find_index_for_key(&entry.0);

            // Move the bucket into the new Array.
            self.arr[index] = Some(entry);
        }
    }

    pub(crate) fn index_from_key<H: Hash + ?Sized>(&self, hashable: &H) -> usize {
        let key_hash = self.hasher.hash_one(hashable);
        (key_hash % self.cap() as u64) as usize
    }

    pub(crate) fn find_index_for_key(&self, key: &K) -> usize {
        let mut index = self.index_from_key(&key);

        // This is where Eq comes in: while there is a value at the current index, but the key
        // isn't equal, increment the index (wrapping at the capacity) and check again.
        // Can't enter recursion unless the load factor is 100%.
        while let Some(existing) = &self.arr[index] && &existing.0 != key {
            index = (index + 1) % self.cap();
        }

        // After that loop, index is either empty or contains an equal key.
        index
    }

    pub(crate) fn find_index_for_borrowed<Q>(&self, key: &Q) -> usize
    where
        // We're introducing a new type parameter here, Q which represents a borrowed version of K
        // where equality and hashing carries over the borrow.
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let mut index = self.index_from_key(&key);

        // This is where Eq comes in: while there is a value at the current index, but the key
        // isn't equal, increment the index (wrapping at the capacity) and check again.
        // Can't enter recursion unless the load factor is 100%.
        while let Some(existing) = &self.arr[index] && existing.0.borrow() != key {
            index = (index + 1) % self.cap();
        }

        // After that loop, index is either empty or contains an equal key.
        index
    }
}

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self {
        HashMap::new()
    }
}