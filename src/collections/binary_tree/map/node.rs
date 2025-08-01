use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::collections::contiguous::Vector;

pub(crate) struct Branch<K: Ord, V>(pub Option<Box<Node<K, V>>>);

pub(crate) struct Node<K: Ord, V> {
    pub left: Branch<K, V>,
    pub right: Branch<K, V>,
    pub key: K,
    pub value: V,
}

impl<K: Ord, V> Node<K, V> {
    pub fn into_tuple(self) -> (K, V) {
        (self.key, self.value)
    }

    pub const fn tuple(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}

impl<K: Ord, V> Branch<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match &mut self.0 {
            Some(node) => match key.cmp(&node.key) {
                Ordering::Less => node.left.insert(key, value),
                Ordering::Greater => node.right.insert(key, value),
                Ordering::Equal => Some(mem::replace(&mut node.value, value)),
            },
            None => {
                self.0 = Some(Box::new(Node {
                    left: None.into(),
                    right: None.into(),
                    key,
                    value,
                }));
                None
            },
        }
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match &mut self.0 {
            Some(node) => match key.cmp(node.key.borrow()) {
                Ordering::Less => node.left.remove_entry(key),
                Ordering::Greater => node.right.remove_entry(key),
                Ordering::Equal => Some(
                    // SAFETY: We've already matched self.0 as a Some, but we need the mutable
                    // reference here.
                    unsafe { mem::take(&mut self.0).unwrap_unchecked().into_tuple() }
                ),
            },
            None => None,
        }
    }

    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match &self.0 {
            Some(node) => match key.cmp(node.key.borrow()) {
                Ordering::Less => node.left.get_entry(key),
                Ordering::Greater => node.right.get_entry(key),
                Ordering::Equal => Some(node.tuple()),
            },
            None => None,
        }
    }

    // pub fn get<Q>(&self, key: &Q) -> Option<&V>
    // where
    //     K: Borrow<Q>,
    //     Q: Ord + ?Sized
    // {
    //     match &self.0 {
    //         Some(node) => match key.cmp(node.key.borrow()) {
    //             Ordering::Less => node.left.get(key),
    //             Ordering::Greater => node.right.get(key),
    //             Ordering::Equal => Some(&node.value),
    //         },
    //         None => None,
    //     }
    // }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match &mut self.0 {
            Some(node) => match key.cmp(node.key.borrow()) {
                Ordering::Less => node.left.get_mut(key),
                Ordering::Greater => node.right.get_mut(key),
                Ordering::Equal => Some(&mut node.value),
            },
            None => None,
        }
    }

    pub fn contains<Q>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match &mut self.0 {
            Some(node) => match key.cmp(node.key.borrow()) {
                Ordering::Less => node.left.contains(key),
                Ordering::Greater => node.right.contains(key),
                Ordering::Equal => true,
            },
            None => false,
        }
    }

    pub fn first_entry(&self) -> Option<(&K, &V)> {
        match &self.0 {
            Some(node) => match node.left.first_entry() {
                Some(e) => Some(e),
                None => Some(node.tuple()),
            },
            None => None,
        }
    }

    pub fn take_first_entry(&mut self) -> Option<(K, V)> {
        match &mut self.0 {
            Some(node) => match node.left.take_first_entry() {
                Some(e) => Some(e),
                None => Some(
                    // SAFETY: We've already matched self.0 as a Some, but we need the mutable
                    // reference here.
                    unsafe { mem::take(&mut self.0).unwrap_unchecked().into_tuple() }
                ),
            },
            None => None,
        }
    }

    pub fn last_entry(&self) -> Option<(&K, &V)> {
        match &self.0 {
            Some(node) => match node.right.last_entry() {
                Some(e) => Some(e),
                None => Some(node.tuple()),
            },
            None => None,
        }
    }

    pub fn take_last_entry(&mut self) -> Option<(K, V)> {
        match &mut self.0 {
            Some(node) => match node.right.take_last_entry() {
                Some(e) => Some(e),
                None => Some(
                    // SAFETY: We've already matched self.0 as a Some, but we need the mutable
                    // reference here.
                    unsafe { mem::take(&mut self.0).unwrap_unchecked().into_tuple() }
                ),
            },
            None => None,
        }
    }
}

impl<K: Ord, V> Deref for Branch<K, V> {
    type Target = Option<Box<Node<K, V>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Ord, V> DerefMut for Branch<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: Ord, V> From<Option<Box<Node<K, V>>>> for Branch<K, V> {
    fn from(value: Option<Box<Node<K, V>>>) -> Self {
        Branch(value)
    }
}

impl<K: Ord + Debug, V: Debug> Debug for Branch<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(node) => write!(
                f,
                "{}\n({:?}: {:?})\n{}",
                format!("{:?}", node.left)
                    .lines()
                    .map(|l| String::from("┌    ") + l)
                    .collect::<Vector<_>>()
                    .join("\n"),
                node.key,
                node.value,
                format!("{:?}", node.right)
                    .lines()
                    .map(|l| String::from("└    ") + l)
                    .collect::<Vector<_>>()
                    .join("\n")
            ),
            None => write!(f, "-"),
        }
    }
}
