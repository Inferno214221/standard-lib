use std::borrow::Borrow;
use std::iter::{Chain, FusedIterator};
use std::marker::PhantomData;
use std::mem;

pub trait SetInterface<T: Borrow<Q>, Q: ?Sized>: Sized {
    /// Returns true if the Set contains `item`.
    fn contains(&self, item: &Q) -> bool;

    /// Returns a reference to the contained element equal to the provided `item` or None if there
    /// isn't one.
    fn get(&self, item: &Q) -> Option<&T>;

    /// Removes `item` from the Set, returning it if it exists.
    fn remove(&mut self, item: &Q) -> Option<T>;
}

pub trait SetIterator<T>: IntoIterator<Item = T> + SetInterface<T, T> + Sized {
    type Iter<'a>: Iterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    /// Returns and iterator over all elements in the HashMap, as references.
    fn iter<'a>(&'a self) -> Self::Iter<'a>;

    /// Creates an owned iterator over all items that are in `self` but not `rhs`. (`self \ rhs`)
    fn into_difference(self, other: Self) -> IntoDifference<Self, T> {
        IntoDifference {
            inner: self.into_iter(),
            other,
            _phantom: PhantomData,
        }
    }

    /// Creates a borrowed iterator over all items that are in `self` but not `rhs`. (`self \ rhs`)
    fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, Self, T> {
        Difference {
            inner: self.iter(),
            other,
        }
    }

    /// Creates an owned iterator over all items that are in `self` or `rhs` but not both. (`self △
    /// rhs`)
    fn into_symmetric_difference(self, other: Self) -> IntoSymmetricDifference<Self, T> {
        IntoSymmetricDifference {
            state: IterAndSet {
                iterator_a: self.into_iter(),
                set_b: Some(other),
                _phantom: PhantomData,
            },
        }
    }

    /// Creates a borrowed iterator over all items that are in `self` or `rhs` but not both. (`self
    /// △ rhs`)
    fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, Self, T> {
        SymmetricDifference {
            inner: self.difference(other).chain(other.difference(self)),
        }
    }

    /// Creates an owned iterator over all items that are in both `self` and `rhs`. (`self ∩ rhs`)
    fn into_intersection(self, other: Self) -> IntoIntersection<Self, T> {
        IntoIntersection {
            inner: self.into_iter(),
            other,
            _phantom: PhantomData,
        }
    }

    /// Creates a borrowed iterator over all items that are in both `self` and `rhs`. (`self ∩ rhs`)
    fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, Self, T> {
        Intersection {
            inner: self.iter(),
            other,
        }
    }

    /// Creates an owned iterator over all items that are in either `self` or `rhs`. (`self ∪
    /// rhs`)
    fn into_union(self, other: Self) -> IntoUnion<Self, T> {
        IntoUnion {
            state: IterAndSet {
                iterator_a: self.into_iter(),
                set_b: Some(other),
                _phantom: PhantomData,
            },
        }
    }

    /// Creates a borrowed iterator over all items that are in either `self` or `rhs`. (`self ∪
    /// rhs`)
    fn union<'a>(&'a self, other: &'a Self) -> Union<'a, Self, T> {
        Union {
            inner: self.iter().chain(other.difference(self)),
        }
    }

    /// Returns true if `other` contains all elements of `self`. (`self ⊆ other`)
    fn is_subset(&self, other: &Self) -> bool {
        other.is_superset(self)
    }

    /// Returns true if `self` contains all elements of `other`. (`self ⊇ other`)
    fn is_superset(&self, other: &Self) -> bool {
        for item in other.iter() {
            if !self.contains(item) {
                return false;
            }
        }
        true
    }
}

pub(crate) enum SetIterToggle<S: SetIterator<T>, T> {
    IterAndSet {
        iterator_a: S::IntoIter,
        // We use Option<S> so that we can move out from behind a mutable reference when
        // switching states. The value can be assumed to always be Some.
        // We're using Option rather than MaybeUninit so that it drops correctly and because it
        // implements Default.
        set_b: Option<S>,
        _phantom: PhantomData<T>,
    },
    OtherSet {
        iterator_b: S::IntoIter,
        _phantom: PhantomData<T>,
    },
}

use SetIterToggle::*;

pub struct IntoDifference<S: SetIterator<T>, T> {
    pub(crate) inner: S::IntoIter,
    pub(crate) other: S,
    // We need the type parameters T for SetIterator, despite not directly owning any Ts.
    pub(crate) _phantom: PhantomData<T>,
}

impl<S: SetIterator<T>, T> Iterator for IntoDifference<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = &next
            && self.other.contains(item)
        {
            next = self.inner.next();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: SetIterator<T>, T> FusedIterator for IntoDifference<S, T> {}

pub struct Difference<'a, S: SetIterator<T>, T: 'a> {
    pub(crate) inner: S::Iter<'a>,
    pub(crate) other: &'a S,
}

impl<'a, S: SetIterator<T>, T: 'a> Iterator for Difference<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = next
            && self.other.contains(item)
        {
            next = self.inner.next();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: SetIterator<T>, T: 'a> FusedIterator for Difference<'a, S, T> {}

pub struct IntoSymmetricDifference<S: SetIterator<T>, T> {
    pub(crate) state: SetIterToggle<S, T>,
}

impl<S: SetIterator<T>, T> Iterator for IntoSymmetricDifference<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IterAndSet { iterator_a, set_b, .. } => {
                let mut next = iterator_a.next();
                while let Some(item) = &next
                    // SAFETY: set_b is Some unless otherwise stated.
                    && unsafe {
                        set_b.as_mut()
                            .unwrap_unchecked()
                            .remove(item)
                            .is_some()
                    }
                {
                    next = iterator_a.next();
                }
                match next {
                    Some(val) => Some(val),
                    None => {
                        // SAFETY: set_b is always Some, except for the duration of this block where
                        // its value is moved.
                        let iterator_b = unsafe {
                            mem::take(set_b)
                                .unwrap_unchecked()
                                .into_iter()
                        };
                        self.state = OtherSet {
                            iterator_b,
                            _phantom: PhantomData,
                        };
                        // Call next once the state is valid.
                        self.next()
                    },
                }
            },
            OtherSet { iterator_b, .. } => iterator_b.next(),
        }
    }

    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     self.inner.size_hint()
    // }
}

impl<S: SetIterator<T>, T> FusedIterator for IntoSymmetricDifference<S, T> {}

pub struct SymmetricDifference<'a, S: SetIterator<T>, T: 'a> {
    pub(crate) inner: Chain<Difference<'a, S, T>, Difference<'a, S, T>>,
}

impl<'a, S: SetIterator<T>, T: 'a> Iterator for SymmetricDifference<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: SetIterator<T>, T: 'a> FusedIterator for SymmetricDifference<'a, S, T> {}

pub struct IntoIntersection<S: SetIterator<T>, T> {
    pub(crate) inner: S::IntoIter,
    pub(crate) other: S,
    pub(crate) _phantom: PhantomData<T>,
}

impl<S: SetIterator<T>, T> Iterator for IntoIntersection<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = &next
            && !self.other.contains(item)
        {
            next = self.inner.next();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: SetIterator<T>, T> FusedIterator for IntoIntersection<S, T> {}

pub struct Intersection<'a, S: SetIterator<T>, T: 'a> {
    pub(crate) inner: S::Iter<'a>,
    pub(crate) other: &'a S,
}

impl<'a, S: SetIterator<T>, T: 'a> Iterator for Intersection<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(item) = next
            && !self.other.contains(item)
        {
            next = self.inner.next();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: SetIterator<T>, T: 'a> FusedIterator for Intersection<'a, S, T> {}

pub struct IntoUnion<S: SetIterator<T>, T> {
    pub(crate) state: SetIterToggle<S, T>,
}

impl<S: SetIterator<T>, T> Iterator for IntoUnion<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IterAndSet { iterator_a, set_b, .. } => {
                let mut next = iterator_a.next();
                while let Some(item) = &next
                    // SAFETY: set_b is Some unless otherwise stated.
                    && unsafe { set_b.as_ref().unwrap_unchecked().contains(item) }
                {
                    next = iterator_a.next();
                }
                match next {
                    Some(val) => Some(val),
                    None => {
                        // SAFETY: set_b is always Some, except for the duration of this block where
                        // its value is moved.
                        let iterator_b = unsafe {
                            mem::take(set_b)
                                .unwrap_unchecked()
                                .into_iter()
                        };
                        self.state = OtherSet {
                            iterator_b,
                            _phantom: PhantomData,
                        };
                        // Call next once the state is valid.
                        self.next()
                    },
                }
            },
            OtherSet { iterator_b, .. } => iterator_b.next(),
        }
    }

    // TODO: validate / update all size_hints.
    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     self.inner.size_hint()
    // }
}

impl<S: SetIterator<T>, T> FusedIterator for IntoUnion<S, T> {}

pub struct Union<'a, S: SetIterator<T>, T: 'a> {
    pub(crate) inner: Chain<S::Iter<'a>, Difference<'a, S, T>>,
}

impl<'a, S: SetIterator<T>, T: 'a> Iterator for Union<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: SetIterator<T>, T: 'a> FusedIterator for Union<'a, S, T> {}
