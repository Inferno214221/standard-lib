use std::iter::FusedIterator;

use super::BinaryTreeMap;

impl<K: Ord, V> IntoIterator for BinaryTreeMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

pub struct IntoIter<K: Ord, V>(BinaryTreeMap<K, V>);

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        // This isn't good, it takes log2 n every time, but the alternative is a doubly linked one.
        self.0.take_first_entry()
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.take_last_entry()
    }
}

impl<K: Ord, V> FusedIterator for IntoIter<K, V> {}

// TODO: write Iter for BinaryTreeMap

impl<'a, K: Ord, V> IntoIterator for &'a BinaryTreeMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

pub struct Iter<'a, K: Ord, V>(&'a BinaryTreeMap<K, V>);

impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // Can't even use the same thing rip
        todo!()
    }
}

impl<'a, K: Ord, V> FusedIterator for Iter<'a, K, V> {}

pub struct IntoKeys<K: Ord, V>(pub(crate) IntoIter<K, V>);

impl<K: Ord, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: Ord, V> FusedIterator for IntoKeys<K, V> {}

pub struct Keys<'a, K: Ord, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K: Ord, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K: Ord, V> FusedIterator for Keys<'a, K, V> {}

pub struct IntoValues<K: Ord, V>(pub(crate) IntoIter<K, V>);

impl<K: Ord, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K: Ord, V> FusedIterator for IntoValues<K, V> {}

// TODO: write ValuesMut for BinaryTreeMap

pub struct Values<'a, K: Ord, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K: Ord, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K: Ord, V> FusedIterator for Values<'a, K, V> {}
