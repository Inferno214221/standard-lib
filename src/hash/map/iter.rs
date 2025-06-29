use std::hash::{BuildHasher, Hash};

use crate::hash::map::{HashMap, Bucket};

use crate::contiguous::array::IntoIter as ArrIntoIter;
use std::slice::IterMut as ArrIterMut;
use std::slice::Iter as ArrIter;

impl<K: Hash + Eq, V, B: BuildHasher> IntoIterator for HashMap<K, V, B> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len(),
            inner: self.arr.into_iter(),
        }
    }
}

pub struct IntoIter<K, V> {
    inner: ArrIntoIter<Bucket<K, V>>,
    len: usize
}

impl<K: Hash + Eq, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        match next {
            Some(Some(entry)) => Some(*entry),
            None => None,
            Some(None) => unreachable!(),
        }
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }
}

impl<'a, K: Hash + Eq, V, B: BuildHasher> IntoIterator for &'a mut HashMap<K, V, B> {
    type Item = &'a mut (K, V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            len: self.len(),
            inner: self.arr.iter_mut(),
        }
    }
}

pub struct IterMut<'a, K, V> {
    inner: ArrIterMut<'a, Bucket<K, V>>,
    len: usize
}

impl<'a, K: Hash + Eq, V> Iterator for IterMut<'a, K, V> {
    type Item = &'a mut (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        match next {
            Some(Some(entry)) => Some(&mut **entry),
            None => None,
            Some(None) => unreachable!(),
        }
    }
}

impl<'a, K: Hash + Eq, V, B: BuildHasher> IntoIterator for &'a HashMap<K, V, B> {
    type Item = &'a (K, V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            len: self.len(),
            inner: self.arr.iter(),
        }
    }
}

pub struct Iter<'a, K, V> {
    inner: ArrIter<'a, Bucket<K, V>>,
    len: usize
}

impl<'a, K: Hash + Eq, V> Iterator for Iter<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner.next();
        while let Some(None) = next {
            next = self.inner.next();
        }

        match next {
            Some(Some(entry)) => Some(&**entry),
            None => None,
            Some(None) => unreachable!(),
        }
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys(self.into_iter())
    }

    pub fn keys<'a>(&'a self) -> Keys<'a, K, V> {
        Keys(self.iter())
    }
}

pub struct IntoKeys<K, V>(IntoIter<K, V>);

impl<K: Hash + Eq, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.0)
    }
}

pub struct Keys<'a, K, V>(Iter<'a, K, V>);

impl<'a, K: Hash + Eq, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| &e.0)
    }
}

impl<K: Hash + Eq, V, B: BuildHasher> HashMap<K, V, B> {
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    pub fn values_mut<'a>(&'a mut self) -> ValuesMut<'a, K, V> {
        ValuesMut(self.iter_mut())
    }

    pub fn values<'a>(&'a self) -> Values<'a, K, V> {
        Values(self.iter())
    }
}

pub struct IntoValues<K, V>(IntoIter<K, V>);

impl<K: Hash + Eq, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.1)
    }
}

pub struct ValuesMut<'a, K, V>(IterMut<'a, K, V>);

impl<'a, K: Hash + Eq, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| &mut e.1)
    }
}

pub struct Values<'a, K, V>(Iter<'a, K, V>);

impl<'a, K: Hash + Eq, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| &e.1)
    }
}