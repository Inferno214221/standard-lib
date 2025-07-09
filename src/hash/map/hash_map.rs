use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};
use std::mem;
use std::{cmp, fmt};

use super::{IntoKeys, IntoValues, Iter, Keys, Values, ValuesMut};
use crate::contiguous::Array;
use crate::util::fmt::DebugRaw;
use crate::util::option::OptionExtension;

const MIN_ALLOCATED_CAP: usize = 2;

const GROWTH_FACTOR: usize = 2;

const LOAD_FACTOR_NUMERATOR: usize = 4;
const LOAD_FACTOR_DENOMINATOR: usize = 5;

/// A map of keys to values which relies on the keys implementing [`Hash`].
///
/// A custom load factor is not supported at this point, with the default being 4/5.
///
/// It is a logic error for keys in a HashMap to be manipulated in a way that changes their hash.
/// Because of this, HashMap's API prevents mutable access to its keys.
///
/// # Time Complexity
/// For this analysis of time complexity, variables are defined as follows:
/// - `n`: The number of items in the HashMap.
///
/// | Method | Complexity |
/// |-|-|
/// | `len` | `O(1)` |
/// | `insert` | `O(1)`**, `O(n)` |
/// | `insert_unchecked` | `O(1)`* |
/// | `get` | `O(1)`* |
/// | `remove` | `O(1)`* |
/// | `contains` | `O(1)`* |
/// | `reserve` | `O(n)`***, `O(1)` |
///
/// \* In the event of a has collision, these functions will take additional time, while a valid
/// / correct location is found. This additional time is kept at a minimum and hash collisions are
/// unlikely especially with a large capacity.
///
/// \** If the HashMap doesn't have enough capacity for the new element, `insert` will take `O(n)`.
/// \* applies as well.
///
/// \*** If the HashMap has enough capacity for the additional items already, `reserve` is `O(1)`.
pub struct HashMap<K: Hash + Eq, V, B: BuildHasher = RandomState> {
    pub(crate) arr: Array<Bucket<K, V>>,
    pub(crate) len: usize,
    pub(crate) hasher: B,
}

pub(crate) type Bucket<K, V> = Option<(K, V)>;

impl<K: Hash + Eq, V, B: BuildHasher + Default> HashMap<K, V, B> {
    /// Creates a new HashMap with capacity 0 and the default value for `B`. Memory will be
    /// allocated when the capacity changes.
    pub fn new() -> HashMap<K, V, B> {
        HashMap {
            arr: Array::new(),
            len: 0,
            hasher: B::default(),
        }
    }

    /// Creates a new HashMap with the provided `cap`acity, allowing insertions without
    /// reallocation. The default hasher will be used.
    pub fn with_cap(cap: usize) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::repeat_default(cap),
            len: 0,
            hasher: B::default(),
        }
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    /// Creates a new HashMap with capacity 0 and the provided `hasher`.
    pub fn with_hasher(hasher: B) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::new(),
            len: 0,
            hasher,
        }
    }

    /// Creates a new HashMap with the provided `cap`acity and `hasher`.
    pub fn with_cap_and_hasher(cap: usize, hasher: B) -> HashMap<K, V, B> {
        HashMap {
            arr: Array::repeat_default(cap),
            len: 0,
            hasher,
        }
    }

    /// Returns the length of the HashMap.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the HashMap contains no entries.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current capacity of the HashMap.
    pub const fn cap(&self) -> usize {
        self.arr.size
    }

    /// Inserts the provided `key`-`value` pair into the HashMap, increasing the capacity if
    /// required. If the key was already associated with a value, the previous value is returned.
    ///
    /// As with the standard library, the key isn't changed if it already exists.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.should_grow() {
            self.grow()
        }

        // UNREACHABLE: We've just grown if necessary.
        let index = self.find_index_for_key(&key).unreachable();

        // The bucket at index is either empty or contains an equal key.
        match &mut self.arr[index] {
            Some(existing) => {
                // Replace the value with the provided one.
                Some(mem::replace(
                    &mut existing.1,
                    value
                ))
            },
            None => {
                // Create a new bucket with the provided values.
                self.arr[index] = Some((key, value));
                self.len += 1;
                None
            },
        }
    }

    /// Inserts the provided `key`-`value` pair without checking if the HashMap has enough capacity.
    /// If the key was already associated with a value, the previous value is returned.
    ///
    /// As with the standard library, the key isn't changed if it already exists.
    ///
    /// # Safety
    /// It is the responsibility of the caller to ensure that the HashMap has enough capacity to add
    /// the provided entry, using methods like [`reserve`][HashMap::reserve] or
    /// [`with_cap`](HashMap::with_cap).
    ///
    /// # Panics
    /// Panics if the HashMap has a capacity of 0, as it isn't possible to find a bucket associated
    /// with the key.
    pub unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Option<V> {
        let index = self.find_index_for_key(&key)
            .expect("Unchecked insertion into HashMap with capacity 0!");

        // The bucket at index is either empty or contains an equal key.
        match &mut self.arr[index] {
            Some(existing) => {
                // Replace the value with the provided one.
                Some(mem::replace(
                    &mut existing.1,
                    value
                ))
            },
            None => {
                // Create a new bucket with the provided values.
                self.arr[index] = Some((key, value));
                self.len += 1;
                None
            },
        }
    }

    /// Returns the entry for the provided `key` as a key-value pair or None if there is no entry.
    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        // We're introducing a new type parameter here, Q which represents a borrowed version of K
        // where equality and hashing carries over the borrow.
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.find_index_for_key(key)?;

        // If the bucket at index is empty, the map doesn't contain the key.
        match &self.arr[index] {
            Some(existing) => Some((&existing.0, &existing.1)),
            None => None,
        }
    }

    /// Returns a reference to the value associated with the provided `key` or None if the map
    /// contains no values for `key`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.find_index_for_key(key)?;

        // If the bucket at index is empty, the map doesn't contain the key.
        match &self.arr[index] {
            Some(existing) => Some(&existing.1),
            None => None,
        }
    }

    /// Returns a mutable reference to the value associated with the provided `key` or None if the
    /// map contains no values for `key`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.find_index_for_key(key)?;

        // If the bucket at index is empty, the map doesn't contain the key.
        match &mut self.arr[index] {
            Some(existing) => Some(&mut existing.1),
            None => None,
        }
    }

    /// Removes the entry associated with `key`, returning it if it exists.
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut index = self.find_index_for_key(key)?;

        // If the bucket at index is empty, the map doesn't contain the key.
        let removed = match mem::take(&mut self.arr[index]) {
            Some(entry) => {
                self.len -= 1;
                Some(entry)
            },
            None => None,
        };

        // UNCHECKED: find_index_for_key returned some, so the cap is not 0.
        let mut next_index = (index + 1) % self.cap();

        // If the next bucket isn't empty and isn't in the correct position, it has been moved to
        // the right at least once. Therefore, move it left once, either putting it in the correct
        // location or improving the proximity to the correct location.
        while let Some(next) = &self.arr[next_index]
            // UNREACHABLE: We've already propagated a None from find_index_for_key, so
            // index_from_key will return Some.
            && self.index_from_key(&next.0).unreachable() != next_index
        {
            let moving = mem::take(&mut self.arr[next_index]);
            let _none = mem::replace(&mut self.arr[index], moving);
            index = next_index;
            // UNCHECKED: find_index_for_key returned some, so the cap is not 0.
            next_index = (index + 1) % self.cap();
        }

        removed
    }

    /// Removes the entry associated with `key`, returning the value if it exists.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Returns true if there is a value associated with the provided `key`.
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.find_index_for_key(key);

        match index {
            Some(i) => self.arr[i].is_some(),
            None => false,
        }
    }

    /// Increases the capacity of the HashMap to ensure that len + `extra` entries will fit without
    /// exceeding the load factor.
    pub fn reserve(&mut self, extra: usize) {
        let new_cap = self.len.strict_add(extra) * LOAD_FACTOR_DENOMINATOR / LOAD_FACTOR_NUMERATOR;
        if new_cap <= self.cap() { return; }

        self.realloc_with_cap(new_cap);
    }

    /// Returns and iterator over all key-value pairs in the HashMap, as references.
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }

    /// Consumes self and returns an iterator over all contained keys.
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys(self.into_iter())
    }

    /// Returns and iterator over all keys in the HashMap, as references.
    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys(self.iter())
    }

    /// Consumes self and returns an iterator over all contained values.
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    /// Returns and iterator over all values in the HashMap, as mutable references.
    pub fn values_mut<'a>(&'a mut self) -> ValuesMut<'a, K, V> {
        ValuesMut {
            len: self.len(),
            inner: self.arr.iter_mut(),
        }
    }

    /// Returns and iterator over all values in the HashMap, as references.
    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values(self.iter())
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    /// Determines whether the HashMap's length exceeds the load capacity, suggesting that it should
    /// grow before inserting new entries.
    pub(crate) const fn should_grow(&self) -> bool {
        self.len >= self.arr.size * LOAD_FACTOR_NUMERATOR / LOAD_FACTOR_DENOMINATOR
    }

    /// Grows the HashMap by the growth factor, ensuring that it can hold additional entries.
    pub(crate) fn grow(&mut self) {
        let new_cap = cmp::max(self.cap() * GROWTH_FACTOR, MIN_ALLOCATED_CAP);

        self.realloc_with_cap(new_cap)
    }

    /// Reallocates the HashMap to have capacity equal to `new_cap`, if doing so wouldn't cause the
    /// map to overload. (There isn't a logical way for the map to shrink and drop entries, so this
    /// isn't allowed.)
    pub(crate) fn realloc_with_cap(&mut self, new_cap: usize) {
        // Can't handle dropping values at this point.
        if new_cap * LOAD_FACTOR_NUMERATOR / LOAD_FACTOR_DENOMINATOR < self.len { return; }

        // Replace the Array first so that we can consume the old Array.
        let old_arr = mem::replace(&mut self.arr, Array::repeat_default(new_cap));

        for entry in old_arr.into_iter().flatten() {
            // UNREACHABLE: If the new capacity is 0, the old_arr has no items and we can't enter
            // this loop.
            let index = self.find_index_for_key(&entry.0).unreachable();

            // Move the bucket into the new Array.
            self.arr[index] = Some(entry);
        }
    }

    /// Calculates the ideal index of a bucket for the provided `hashable` (or None if the HashMap
    /// has 0 capacity). This method doesn't consider hash collisions, see
    /// [`HashMap::find_index_for_key`] for that functionality.
    pub(crate) fn index_from_key<H: Hash + ?Sized>(&self, hashable: &H) -> Option<usize> {
        let key_hash = self.hasher.hash_one(hashable);
        key_hash.checked_rem(self.cap() as u64).map(|i| i as usize)
    }

    /// Finds the first valid index for the provided `key` (or None if the HashMap has 0 capacity).
    /// This is done by calculating the ideal index and then iterating until a bucket is found that
    /// is empty or has an equal key.
    pub(crate) fn find_index_for_key<Q>(&self, key: &Q) -> Option<usize>
    where
        // We're introducing a new type parameter here, Q which represents a borrowed version of K
        // where equality and hashing carries over the borrow.
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut index = self.index_from_key(&key)?;

        // This is where Eq comes in: while there is a value at the current index, but the key
        // isn't equal, increment the index (wrapping at the capacity) and check again.
        // Can't enter recursion unless the load factor is 100%.
        while let Some(existing) = &self.arr[index]
            && existing.0.borrow() != key
        {
            // UNCHECKED: find_index_for_key returned some, so the cap is not 0.
            index = (index + 1) % self.cap();
        }

        // After that loop, index is either empty or contains an equal key.
        Some(index)
    }
}

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self {
        HashMap::new()
    }
}

impl<K: Hash + Eq + Debug, V: Debug, B: BuildHasher + Debug> Debug for HashMap<K, V, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashMap")
            .field_with("buckets", |f| f.debug_list().entries(
                self.arr.iter()
                    .map(|o| DebugRaw(match o {
                        Some((k, v)) => format!("({k:?}: {v:?})"),
                        None => "-".into(),
                    }))
            ).finish())
            .field("len", &self.len)
            .field("cap", &self.cap())
            .field("hasher", &self.hasher)
            .finish()
    }
}

impl<K: Hash + Eq + Debug, V: Debug, B: BuildHasher> Display for HashMap<K, V, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#")?;
        f.debug_map().entries(self.iter()).finish()
    }
}
