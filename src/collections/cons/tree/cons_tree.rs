use std::{fmt::{self, Debug}, mem, ops::Deref, rc::Rc};

use super::{Iter, OwnedIter, RcIter, UniqueIter};

/// A references counted, linked list, implemented similar to a cons list. This type is useful as an
/// immutable list with cheap, shallow cloning, that can share nodes with other instances.
///
/// When cloned, only the 'head' of the tree is cloned (just an Rc), with all of the elements
/// included in both list. As a result, the data structure is only mutable from the head, where
/// elements can be [`push`](Self::push)ed or [`pop`](Self::pop_to_owned)ped. This cheap cloning is
/// helpful for implementing procedures such that include rollbacks or branching.
#[derive(Clone)]
pub struct ConsTree<T> {
    pub(crate) inner: Option<Rc<ConsTreeNode<T>>>,
}

#[derive(Clone)]
pub struct ConsTreeNode<T> {
    pub value: T,
    pub(crate) next: ConsTree<T>,
}

impl<T> ConsTree<T> {
    /// Creates a new, empty `ConsTree`.
    pub const fn new() -> ConsTree<T> {
        ConsTree {
            inner: None
        }
    }

    /// Pushes a new element onto the start of this `ConsTree`, updating this list's head without
    /// affecting any overlapping lists (shallow clones).
    pub fn push(&mut self, value: T) {
        let old = mem::take(&mut self.inner);

        self.inner = Some(Rc::new(ConsTreeNode {
            value,
            next: ConsTree {
                inner: old
            },
        }))
    }
}

impl<T> ConsTree<T> {
    /// Returns `true` if this `ConsTree` contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    /// Produces a borrowed iterator over all elements in this list, both unique and shared.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.inner.as_deref(),
        }
    }

    /// Produces an iterator over all of the underlying [`Rc`] instances in this list.
    pub const fn into_iter_rc(self) -> RcIter<T> {
        RcIter {
            inner: self,
        }
    }

    /// Returns `true` if this entire list is unique (doesn't share any items with another list).
    ///
    /// If this method returns true, [`into_iter_unique`](Self::into_iter_unique) will produce every
    /// item in the list.
    pub fn is_unique(&self) -> bool {
        let mut next = &self.inner;
        while let Some(node) = next {
            if !is_unqiue(node) {
                return false;
            }
            next = &node.next.inner;
        }
        true
    }

    /// Returns `true` if the head element of this list is unique.
    pub fn is_head_unique(&self) -> bool {
        match &self.inner {
            Some(node) => is_unqiue(node),
            None => true,
        }
    }

    /// Pops the head element of this list, if it is unique. Otherwise, `self` remains unchanged.
    pub fn pop_if_unique(&mut self) -> Option<T> {
        let inner = mem::take(&mut self.inner);

        match inner {
            Some(rc) => {
                // If this returns here, self.inner.inner has been replaced with a None.
                let ConsTreeNode { value, next } = Rc::into_inner(rc)?;
                self.inner = next.inner;
                Some(value)
            },
            None => {
                self.inner = inner;
                None
            },
        }
    }

    /// Removes all shared items from this list and returns them as another `ConsTree`. After
    /// calling this method, `self` is guaranteed to be unique.
    pub fn keep_unique(&mut self) -> ConsTree<T> {
        let mut node = match &mut self.inner {
            Some(inner) => inner,
            // The list is empty.
            None => return ConsTree::new(),
        };

        let mut node_mut = match Rc::get_mut(node) {
            Some(node_mut) => node_mut,
            // The list contains items, but none are uniquely referenced.
            None => return ConsTree::new(),
        };

        loop {
            // We need to borrow the next node once once as a reference and then conditionally, as a
            // mutable reference.
            if let Some(true) = node_mut.next.inner.as_ref().map(is_unqiue) {
                // We know that node_mut.next.inner is Some, but we couldn't borrow it as a
                // mutable reference until we knew it was unique.
                node = node_mut.next.inner.as_mut().unwrap();
            } else {
                // node_mut.next.inner is either None or a non unique node. For both cases, we take
                // it and return it as the non_unique component.
                return ConsTree {
                    inner: mem::take(&mut node_mut.next.inner)
                };
            }

            // We've already checked that the node is unqiue, and diverged otherwise.
            // Rc is !Sync, so we don't have to worry about TOCTOU.
            node_mut = Rc::get_mut(node).unwrap();
        }
    }

    /// Produces an iterator over all of the elements in this list, until a shared element is found.
    ///
    /// This iterator does no cloning and produces owned items, by completely ignoring any items
    /// that are shared by other lists.
    ///
    /// If called on every clone of a single initial `ConsTree`, every element of the tree will be
    /// returned by an iterator only once.
    pub const fn into_iter_unique(self) -> UniqueIter<T> {
        UniqueIter {
            inner: self,
        }
    }
}

impl<T: Clone> ConsTree<T> {
    /// Pops the head element from this list, cloning if it is shared by another `ConsTree`.
    /// Regardless of if a clone is required, the head of this list will be updated.
    pub fn pop_to_owned(&mut self) -> Option<T> {
        let inner = mem::take(&mut self.inner);

        match inner {
            Some(node) => {
                let ConsTreeNode { value, next } = Rc::unwrap_or_clone(node);
                self.inner = next.inner;
                Some(value)
            },
            None => {
                self.inner = inner;
                None
            },
        }
    }

    /// Produces an iterator over all elements in this list, returning owned items by cloning any
    /// shared elements.
    pub const fn into_iter_owned(self) -> OwnedIter<T> {
        OwnedIter {
            inner: self,
        }
    }

    /// Produces a deep clone of this `ConsTree`. The result has a clone of every element in this
    /// list, without sharing any. The result is unique.
    pub fn deep_clone(&self) -> ConsTree<T> {
        let refs: Vec<_> = self.iter().collect();

        refs.into_iter()
            .rev()
            .cloned()
            .collect()
    }
}

fn is_unqiue<T>(value: &Rc<T>) -> bool {
    Rc::strong_count(value) == 1
}

impl<T> Default for ConsTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for ConsTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = ConsTree::new();
        for item in iter {
            res.push(item);
        }
        res
    }
}

impl<T> Deref for ConsTreeNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Debug> Debug for ConsTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Some(node) => write!(f, "{:?}", node),
            None => write!(f, "()"),
        }
    }
}

impl<T: Debug> Debug for ConsTreeNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.next.inner {
            Some(node) => write!(f, "({:?}->{:?})", self.value, node),
            None => write!(f, "({:?})", self.value),
        }
    }
}