use std::iter::FusedIterator;

use crate::binary_tree::map::BinaryTreeMap;

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

impl<K: Ord, V> FusedIterator for IntoIter<K, V> {}

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