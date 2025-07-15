use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, RandomState};
use std::iter::TrustedLen;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use super::Iter;
use crate::contiguous::Vector;
use crate::hash::HashMap;
use crate::traits::Set;
use crate::util::fmt::DebugRaw;

/// A set of values that prevents duplicates with the help of the [`Hash`] trait.
///
/// Relies on [`HashMap`] internally, see documentation there for additional details.
///
/// A custom load factor is not supported at this point, with the default being 4/5.
///
/// It is a logic error for keys in a HashSet to be manipulated in a way that changes their hash.
/// Because of this, HashSet's API prevents mutable access to its keys.
///
/// # Time Complexity
/// See [`HashMap`] with the following additions.
///
/// Variables are defined as follows:
/// - `n`: The number of items in the HashSet.
/// - `m`: The number of items in the second HashSet.
///
/// | Method | Complexity |
/// |-|-|
/// | `difference`*, `-` | `O(n)` |
/// | `-=` | `O(m)` |
/// | `symmetric_difference`*, `^` | `O(n+m)` |
/// | `^=` | `O(n+m)`**, `O(m)` |
/// | `intersection`*, `&` | `O(n)` |
/// | `&=` | `O(n)` |
/// | `union`*, `\|` | `O(n+m)` |
/// | `\|=` | `O(n+m)`**, `O(m)` |
/// | `is_subset` | `O(m)` |
/// | `is_superset` | `O(n)` |
///
/// In the event of a has collision, all methods will take additional time. This additional time is
/// kept at a minimum and hash collisions are unlikely especially with a large capacity.
///
/// \* When exhausted.
///
/// \** If the HashMap already has capacity for the additional items, these methods will take `O(m)`
/// instead.
pub struct HashSet<T: Hash + Eq, B: BuildHasher = RandomState> {
    // Yay, we get to do the thing where unit type evaluates to a no-op.
    pub(crate) inner: HashMap<T, (), B>,
}

impl<T: Hash + Eq, B: BuildHasher + Default> HashSet<T, B> {
    /// Creates a new HashSet with capacity 0 and the default value for `B`. Memory will be
    /// allocated when the capacity changes.
    pub fn new() -> HashSet<T, B> {
        HashSet {
            inner: HashMap::new(),
        }
    }

    /// Creates a new HashSet with the provided `cap`acity, allowing insertions without
    /// reallocation. The default hasher will be used.
    pub fn with_cap(cap: usize) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_cap(cap),
        }
    }
}

impl<T: Hash + Eq, B: BuildHasher> HashSet<T, B> {
    /// Creates a new HashSet with capacity 0 and the provided `hasher`.
    pub fn with_hasher(hasher: B) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_hasher(hasher),
        }
    }

    /// Creates a new HashSet with the provided `cap`acity and `hasher`.
    pub fn with_cap_and_hasher(cap: usize, hasher: B) -> HashSet<T, B> {
        HashSet {
            inner: HashMap::with_cap_and_hasher(cap, hasher),
        }
    }

    /// Returns the length of the HashSet (the number of elements it contains).
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the HashSet contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the current capacity of the HashSet.
    pub const fn cap(&self) -> usize {
        self.inner.cap()
    }

    /// Inserts the provided item into the HashSet, increasing its capacity if required. If the item
    /// was already included, no change is made and the method returns false. In other words, the
    /// method returns true if the insertion changes the HashSet.
    pub fn insert(&mut self, item: T) -> bool {
        if self.inner.should_grow() {
            self.inner.grow()
        }

        // SAFETY: We've just grown if necessary.
        let index = unsafe { self.inner.find_index_for_key(&item).unwrap_unchecked() };

        // The Bucket at index is either empty or contains an equal item.
        match self.inner.arr[index] {
            Some(_) => false,
            None => {
                // Create a new Bucket with the provided values.
                self.inner.arr[index] = Some((item, ()));
                self.inner.len += 1;
                true
            },
        }
    }

    /// Inserts the provided item, without checking if the HashSet has enough capacity. If the item
    /// was already included, no change is made and the method returns false.
    /// 
    /// # Safety
    /// It is the responsibility of the caller to ensure that the HashSet has enough capacity to add
    /// the provided item, using methods like [`reserve`][HashSet::reserve] or
    /// [`with_cap`](HashSet::with_cap).
    ///
    /// # Panics
    /// Panics if the HashSet has a capacity of 0, as it isn't possible to find a bucket associated
    /// with the item.
    pub unsafe fn insert_unchecked(&mut self, item: T) -> bool {
        let index = self.inner.find_index_for_key(&item)
            .expect("Unchecked insertion into HashSet with capacity 0!");

        // The Bucket at index is either empty or contains an equal item.
        match &mut self.inner.arr[index] {
            Some(_) => true,
            None => {
                // Create a new Bucket with the provided values.
                self.inner.arr[index] = Some((item, ()));
                self.inner.len += 1;
                false
            },
        }
    }

    /// Returns a reference to the contained element equal to the provided `item` or None if there
    /// isn't one.
    pub fn get<Q>(&self, item: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.get_entry(item).map(|(k, _)| k)
    }

    /// Removes `item` from the HashSet, returning it if it exists.
    pub fn remove<Q>(&mut self, item: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.remove_entry(item).map(|(k, _)| k)
    }

    /// Returns true if the HashSet contains `item`.
    pub fn contains<Q>(&self, item: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.inner.contains(item)
    }

    /// Increases the capacity of the HashSet to ensure that len + `extra` elements will fit without
    /// exceeding the load factor.
    pub fn reserve(&mut self, extra: usize) {
        self.inner.reserve(extra)
    }
}

impl<T: Hash + Eq, B: BuildHasher> Set<T> for HashSet<T, B> {
    type Iter<'a> = Iter<'a, T> where Self: 'a;

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.into_iter()
    }

    fn contains(&self, item: &T) -> bool {
        self.contains(item)
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> BitOr for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    /// Returns the union of `self` and `rhs`, as a HashSet.
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitOrAssign for HashSet<T, B> {
    /// Adds all items from `rhs` to `self` to form a union in place.
    fn bitor_assign(&mut self, rhs: Self) {
        self.reserve(rhs.cap());
        for item in rhs {
            // SAFETY: self has capacity for all items in rhs because we just called reserve.
            unsafe { self.insert_unchecked(item); }
        }
    }
}

impl<T: Hash + Eq + Clone, B: BuildHasher + Default> BitAnd for &HashSet<T, B> {
    type Output = HashSet<T, B>;

    /// Returns the intersection of `self` and `rhs`, as a HashSet.
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitAndAssign for HashSet<T, B> {
    /// Removes all items not in `rhs` from `self` to form an intersection in place.
    fn bitand_assign(&mut self, rhs: Self) {
        let mut to_remove = Vector::with_cap(self.cap());
        for item in self.iter() {
            if !rhs.contains(item) {
                to_remove.push(
                    // SAFETY: We are in a loop over self, so cap > 0.
                    unsafe { self.inner.find_index_for_key(item).unwrap_unchecked() }
                );
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

    /// Returns the symmetric difference of `self` and `rhs`, as a HashSet.
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> BitXorAssign for HashSet<T, B> {
    /// Removes all items in both `rhs` and `self` from `self` to form the symmetric difference of
    /// the two in place.
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

    /// Returns the difference of `self` and `rhs`, as a HashSet.
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Hash + Eq, B: BuildHasher> SubAssign for HashSet<T, B> {
    /// Removes all items in `rhs` from `self` to form the difference of the two in place.
    fn sub_assign(&mut self, rhs: Self) {
        for item in rhs {
            self.remove(&item);
        }
    }
}

impl<T: Hash + Eq, B: BuildHasher + Default> Default for HashSet<T, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq, B: BuildHasher> Extend<T> for HashSet<T, B> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        for item in iter {
            self.insert(item);
        }
    }

    fn extend_one(&mut self, item: T) {
        self.insert(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    unsafe fn extend_one_unchecked(&mut self, item: T)
    where
        Self: Sized,
    {
        // SAFETY: extend_reserve is implemented correctly, so all other safety requirements are the
        // responsibility of the caller.
        unsafe { self.insert_unchecked(item); }
    }
}

impl<T, B, I> From<I> for HashSet<T, B>
where
    T: Hash + Eq,
    B: BuildHasher + Default,
    I: Iterator<Item = T> + ExactSizeIterator + TrustedLen,
{
    fn from(value: I) -> Self {
        let iter = value.into_iter();
        let mut set = HashSet::with_cap(iter.len());

        for item in iter {
            // SAFETY: HashSet has been created with the right capacity.
            unsafe { set.insert_unchecked(item); }
        }

        set
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

impl<T: Hash + Eq, B: BuildHasher> PartialEq for HashSet<T, B> {
    /// Two HashSets are considered equal if they contain exactly the same elements.
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.is_subset(other)
            && self.is_superset(other)
    }
}

impl<T: Hash + Eq, B: BuildHasher> Eq for HashSet<T, B> {}

impl<T: Hash + Eq + Debug, B: BuildHasher + Debug> Debug for HashSet<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashSet")
            .field_with("buckets", |f| f.debug_list().entries(
                self.inner.arr.iter()
                    .map(|o| DebugRaw(match o {
                        Some((t, _)) => format!("{t:?}"),
                        None => "-".into(),
                    }))
            ).finish())
            .field("len", &self.len())
            .field("cap", &self.cap())
            .field("hasher", &self.inner.hasher)
            .finish()
    }
}

impl<T: Hash + Eq + Debug, B: BuildHasher> Display for HashSet<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#")?;
        f.debug_set().entries(self.iter()).finish()
    }
}
