use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};
use std::iter::TrustedLen;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use crate::contiguous::Vector;
use crate::hash::set::{Difference, Intersection, SymmetricDifference, Union};
use crate::hash::HashMap;
use super::Iter;

pub struct HashSet<T: Hash + Eq, B: BuildHasher = RandomState> {
    // Yay, we get to do the thing where unit type evaluates to a no-op.
    pub(crate) inner: HashMap<T, (), B>
}

impl<T: Hash + Eq, B: BuildHasher + Default> HashSet<T, B> {
    pub fn new() -> HashSet<T, B> {
        HashSet {
            inner: HashMap::new()
        }
    }

    pub fn with_cap(cap: usize) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_cap(cap)
        }
    }
}

impl<T: Hash + Eq, B: BuildHasher> HashSet<T, B> {
    pub fn with_hasher(hasher: B) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_hasher(hasher)
        }
    }

    pub fn with_cap_and_hasher(cap: usize, hasher: B) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_cap_and_hasher(cap, hasher)
        }
    }

    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub const fn cap(&self) -> usize {
        self.inner.cap()
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.inner.should_grow() {
            self.inner.grow()
        }
        
        let index = self.inner.find_index_for_key(&item);

        // The Bucket at index is either empty or contains an equal item.
        match &mut self.inner.arr[index] {
            Some(_) => {
                false
            },
            None => {
                // Create a new Bucket with the provided values.
                self.inner.arr[index] = Some((item, ()));
                self.inner.len += 1;
                true
            },
        }
    }

    pub unsafe fn insert_unchecked(&mut self, item: T) -> bool {        
        let index = self.inner.find_index_for_key(&item);

        // The Bucket at index is either empty or contains an equal item.
        match &mut self.inner.arr[index] {
            Some(_) => {
                true
            },
            None => {
                // Create a new Bucket with the provided values.
                self.inner.arr[index] = Some((item, ()));
                self.inner.len += 1;
                false
            },
        }
    }

    pub fn remove<Q>(&mut self, item: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.inner.remove_entry(item).map(|e| e.0)
    }

    pub fn contains<Q>(&self, item: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.inner.contains(item)
    }

    pub fn reserve(&mut self, extra: usize) {
        self.inner.reserve(extra)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn difference<'a>(&'a self, other: &'a HashSet<T, B>) -> Difference<'a, T, B> {
        Difference {
            inner: self.iter(),
            other
        }
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a HashSet<T, B>) -> SymmetricDifference<'a, T, B> {
        SymmetricDifference {
            inner: self.difference(other).chain(other.difference(self)),
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a HashSet<T, B>) -> Intersection<'a, T, B> {
        Intersection {
            inner: self.iter(),
            other
        }
    }

    pub fn union<'a>(&'a self, other: &'a HashSet<T, B>) -> Union<'a, T, B> {
        Union {
            inner: self.iter().chain(other.difference(self)),
        }
    }

    pub fn is_subset(&self, other: &HashSet<T, B>) -> bool {
        for item in other {
            if !self.contains(item) {
                return false;
            }
        }
        true
    }

    pub fn is_superset(&self, other: &HashSet<T, B>) -> bool {
        other.is_subset(self)
    }
}

impl<T: Hash + Eq> Default for HashSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, B, I> From<I> for HashSet<T, B>
where
    T: Hash + Eq,
    B: BuildHasher + Default,
    I: Iterator<Item = T> + ExactSizeIterator + TrustedLen
{
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let mut vec = HashSet::with_cap(iter.len());

        for item in iter {
            // SAFETY: Vec has been created with the right capacity.
            unsafe { vec.insert_unchecked(item); }
        }

        vec
    }
}

impl<T: Hash + Eq, B: BuildHasher + Default> FromIterator<T> for HashSet<T, B> {
    fn from_iter<I: IntoIterator<Item = T>>(value: I) -> Self {
        let iter = value.into_iter();
        let mut set = HashSet::with_cap(iter.size_hint().0);

        for item in iter {
            set.insert(item);
        }

        set
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> BitOr for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitOrAssign for HashSet<T, B> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.reserve(rhs.cap());
        for item in rhs {
            unsafe { self.insert_unchecked(item); }
        }
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> BitAnd for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitAndAssign for HashSet<T, B> {
    fn bitand_assign(&mut self, rhs: Self) {
        let mut to_remove = Vector::with_cap(self.cap());
        for item in self.iter() {
            if !rhs.contains(item) {
                to_remove.push(self.inner.find_index_for_key(item));
            }
        }
        for index in to_remove {
            if self.inner.arr[index].is_some() {
                self.inner.arr[index] = None;
                self.inner.len -= 1;
            }
        }
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> BitXor for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitXorAssign for HashSet<T, B> {
    fn bitxor_assign(&mut self, rhs: Self) {
        for item in rhs {
            if self.remove(&item).is_none() {
                self.insert(item);
            }
        }
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> Sub for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> SubAssign for HashSet<T, B> {
    fn sub_assign(&mut self, rhs: Self) {
        for item in rhs {
            self.remove(&item);
        }
    }
}

impl<T: Hash + Eq + Debug, B: BuildHasher + Debug> Debug for HashSet<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashSet")
            .field_with("contents", |f| write!(
                f, "#{{{}}}",
                self.iter()
                    .map(|i| format!("{i:?}"))
                    .collect::<Vector<String>>()
                    .join(", ")
            ))
            .field("len", &self.len())
            .field("cap", &self.cap())
            .field("hasher", &self.inner.hasher)
            .finish()
    }
}

impl<T: Hash + Eq + Display, B: BuildHasher> Display for HashSet<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "#{{{}}}",
            self.iter()
                .map(|i| format!("{i}"))
                .collect::<Vector<String>>()
                .join(", ")
        )
    }
}