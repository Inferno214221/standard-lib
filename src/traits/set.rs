use std::iter::{Chain, FusedIterator};
use std::marker::PhantomData;

pub trait Set<T>: IntoIterator<Item = T> + Sized {
    type Iter<'a>: Iterator<Item = &'a T> where Self: 'a, T: 'a;

    fn contains(&self, item: &T) -> bool;

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

    // /// Creates an owned iterator over all items that are in `self` or `rhs` but not both. (`self △
    // /// rhs`)
    // fn into_symmetric_difference(self, other: Self) -> IntoSymmetricDifference<Self, T> {
    //     IntoSymmetricDifference {
    //         inner: self.into_difference(other).chain(other.into_difference(self)),
    //     }
    // }

    /// Creates a borrowed iterator over all items that are in `self` or `rhs` but not both. (`self
    /// △ rhs`)
    fn symmetric_difference<'a>(
        &'a self,
        other: &'a Self,
    ) -> SymmetricDifference<'a, Self, T> {
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

    // /// Creates an owned iterator over all items that are in either `self` or `rhs`. (`self ∪
    // /// rhs`)
    // fn into_union(self, other: Self) -> IntoUnion<Self, T> {
    //     IntoUnion {
    //         inner: self.iter().chain(other.difference(self)),
    //     }
    // }

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

pub struct IntoDifference<S: Set<T>, T> {
    pub(crate) inner: S::IntoIter,
    pub(crate) other: S,
    // We need the type parameter T for Set, despite not directly owning any T.
    pub(crate) _phantom: PhantomData<T>,
}

impl<S: Set<T>, T> Iterator for IntoDifference<S, T> {
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

impl<S: Set<T>, T> FusedIterator for IntoDifference<S, T> {}

pub struct Difference<'a, S: Set<T>, T: 'a> {
    pub(crate) inner: S::Iter<'a>,
    pub(crate) other: &'a S,
}

impl<'a, S: Set<T>, T: 'a> Iterator for Difference<'a, S, T> {
    type Item = &'a T;

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

impl<'a, S: Set<T>, T: 'a> FusedIterator for Difference<'a, S, T> {}

// FIXME: Can't chain IntoDifference because it requires double ownership.
pub struct IntoSymmetricDifference<S: Set<T>, T> {
    pub(crate) inner: Chain<IntoDifference<S, T>, IntoDifference<S, T>>,
}

impl<S: Set<T>, T> Iterator for IntoSymmetricDifference<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Set<T>, T> FusedIterator for IntoSymmetricDifference<S, T> {}

pub struct SymmetricDifference<'a, S: Set<T>, T: 'a> {
    pub(crate) inner: Chain<Difference<'a, S, T>, Difference<'a, S, T>>,
}

impl<'a, S: Set<T>, T: 'a> Iterator for SymmetricDifference<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: Set<T>, T: 'a> FusedIterator for SymmetricDifference<'a, S, T> {}

pub struct IntoIntersection<S: Set<T>, T> {
    pub(crate) inner: S::IntoIter,
    pub(crate) other: S,
    // We need the type parameter T for Set, despite not directly owning any T.
    pub(crate) _phantom: PhantomData<T>,
}

impl<S: Set<T>, T> Iterator for IntoIntersection<S, T> {
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

impl<S: Set<T>, T> FusedIterator for IntoIntersection<S, T> {}

pub struct Intersection<'a, S: Set<T>, T: 'a> {
    pub(crate) inner: S::Iter<'a>,
    pub(crate) other: &'a S,
}

impl<'a, S: Set<T>, T: 'a> Iterator for Intersection<'a, S, T> {
    type Item = &'a T;

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

impl<'a, S: Set<T>, T: 'a> FusedIterator for Intersection<'a, S, T> {}

// FIXME: Can't chain IntoIter and IntoDifference because it requires double ownership.
pub struct IntoUnion<S: Set<T>, T> {
    pub(crate) inner: Chain<S::IntoIter, IntoDifference<S, T>>,
}

impl<S: Set<T>, T> Iterator for IntoUnion<S, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Set<T>, T> FusedIterator for IntoUnion<S, T> {}

pub struct Union<'a, S: Set<T>, T: 'a> {
    pub(crate) inner: Chain<S::Iter<'a>, Difference<'a, S, T>>,
}

impl<'a, S: Set<T>, T: 'a> Iterator for Union<'a, S, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, S: Set<T>, T: 'a> FusedIterator for Union<'a, S, T> {}
