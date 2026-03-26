use std::{fmt::{self, Debug}, mem, ops::Deref, rc::Rc};

use super::{Iter, OwnedIter, RcIter, UniqueIter};

/// A references counted, linked list implemented similar to a cons list. This type is useful as an
/// list of immutable items with cheap, shallow cloning, that can share nodes with other instances.
///
/// When cloned, only the 'head' of the tree is cloned (just an Rc), with all of the elements
/// included in both list. As a result, the data structure is only mutable from the head, where
/// elements can be [`push`](Self::push)ed or [`pop`](Self::pop_to_owned)ped. This cheap cloning is
/// helpful for implementing procedures such that include rollbacks or branching.
pub struct ConsBranch<T> {
    pub(crate) inner: Option<Rc<ConsNode<T>>>,
}

/// Largely intended as an internal type, these nodes are returned by [`ConsBranch::into_iter_rc`]
/// because the interior of the [`Rc`] can't be unwrapped in place.
///
/// To help using these nodes, a couple of useful traits have been implemented:
/// - [`Deref<Target = T> for ConsNode<T>`](Deref) for accessing the contained value.
/// - [`Into<ConsBranch> for Rc<ConsNode<T>>`](Into) for creating a new [`ConsBranch`] from an
/// [`Rc`].
///
/// Note that cloning a `ConsNode` directly is _not_ cheap as it is with [`ConsBranch`] because
/// the node contains the value (of type `T`) itself.
#[derive(Clone)]
pub struct ConsNode<T> {
    pub(crate) value: T,
    pub(crate) next: ConsBranch<T>,
}

impl<T> ConsBranch<T> {
    /// Creates a new, empty `ConsBranch`.
    pub const fn new() -> ConsBranch<T> {
        ConsBranch {
            inner: None
        }
    }

    /// Pushes a new element onto the start of this `ConsBranch`, updating this list's head without
    /// affecting any overlapping lists (shallow clones).
    pub fn push(&mut self, value: T) {
        let old = mem::take(&mut self.inner);

        self.inner = Some(Rc::new(ConsNode {
            value,
            next: ConsBranch {
                inner: old
            },
        }))
    }

    /// Returns a reference to the element contained within the head node.
    pub fn get_head(&self) -> Option<&T> {
        self.inner.as_deref().map(ConsNode::deref)
    }

    /// Returns `true` if this `ConsBranch` contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    /// Produces a borrowed [`Iterator<Item = &T>`](Iter) over all elements in this list, both
    /// unique and shared.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.inner.as_deref(),
        }
    }

    /// Produces an [`Iterator<Item = Rc<ConsNode<T>>>`](RcIter) over all of the underlying [`Rc`]
    /// instances in this list.
    ///
    /// Each referenced element is considered 'shared' for the lifetime of the [`Rc`] produced by
    /// this iterator. To return a `ConsBranch` to being unique, this iterator and all produced
    /// [`Rc`]s need to be dropped.
    pub fn into_iter_rc(&self) -> RcIter<T> {
        RcIter {
            // We clone here because we are also cloning every step, there is no point taking an
            // owned self.
            inner: self.clone(),
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
        if Rc::get_mut(self.inner.as_mut()?).is_none() {
            return None;
        }

        // We've just confirmed that self.inner is Some and that it is unique.
        let inner = mem::take(&mut self.inner).unwrap();
        let ConsNode { value, next } = Rc::into_inner(inner).unwrap();
        self.inner = next.inner;

        Some(value)
    }

    /// Removes all unique items from this list and returns them as another `ConsBranch`.
    pub fn split_off_unique(&mut self) -> ConsBranch<T> {
        let mut node = match &mut self.inner {
            Some(inner) => inner,
            // The list is empty.
            None => return ConsBranch::new(),
        };

        let mut node_mut = match Rc::get_mut(node) {
            Some(node_mut) => node_mut,
            // The list contains items, but none are uniquely referenced.
            None => return ConsBranch::new(),
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
                // it as the shared head of the tree and replace it with a None.
                let shared_head = mem::take(&mut node_mut.next.inner);
                // We then replace the inner value with it, leaving self as entirely shared and
                // returning the head of the unique portion.
                return ConsBranch {
                    inner: mem::replace(&mut self.inner, shared_head),
                };
            }

            // We've already checked that the node is unqiue, and diverged otherwise.
            // Rc is !Sync, so we don't have to worry about TOCTOU.
            node_mut = Rc::get_mut(node).unwrap();
        }
    }

    /// Produces an [`Iterator<Item = T>`](UniqueIter) over the elements in this list, producing
    /// owned values until a shared element is found.
    ///
    /// This iterator does no cloning and produces owned items by completely ignoring any elements
    /// that are shared by other lists.
    ///
    /// If called on every clone of a single initial `ConsBranch`, every element of the tree will be
    /// returned by an iterator only once.
    pub const fn into_iter_unique(self) -> UniqueIter<T> {
        UniqueIter {
            inner: self,
        }
    }
}

impl<T: Clone> ConsBranch<T> {
    /// Pops the head element from this list, cloning if it is shared by another `ConsBranch`.
    /// Regardless of if a clone is required, the head of this list will be updated.
    pub fn pop_to_owned(&mut self) -> Option<T> {
        let inner = mem::take(&mut self.inner);

        match inner {
            Some(node) => {
                let ConsNode { value, next } = Rc::unwrap_or_clone(node);
                self.inner = next.inner;
                Some(value)
            },
            None => {
                self.inner = inner;
                None
            },
        }
    }

    /// Produces an [`Iterator<Item = T>`](OwnedIter) over all elements in this list, returning
    /// owned items by cloning any shared elements.
    pub const fn into_iter_owned(self) -> OwnedIter<T> {
        OwnedIter {
            inner: self,
        }
    }

    /// Produces a deep clone of this `ConsBranch`. The result has a clone of every element in this
    /// list, without sharing any. The result is unique.
    pub fn deep_clone(&self) -> ConsBranch<T> {
        let refs: Vec<_> = self.iter().collect();

        refs.into_iter()
            .rev()
            .cloned()
            .collect()
    }
}

impl<T> Clone for ConsBranch<T> {
    /// Creates a cheap (shallow) clone of this `ConsBranch`, with all the same underlying elements.
    /// After cloning, all elements of the list are considered 'shared' between the original list
    /// and the clone.
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}

fn is_unqiue<T>(value: &Rc<T>) -> bool {
    Rc::strong_count(value) == 1
}

impl<T> Default for ConsBranch<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for ConsBranch<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = ConsBranch::new();
        for item in iter {
            res.push(item);
        }
        res
    }
}

impl<T> From<Rc<ConsNode<T>>> for ConsBranch<T> {
    fn from(value: Rc<ConsNode<T>>) -> Self {
        ConsBranch {
            inner: Some(value),
        }
    }
}

impl<T> Deref for ConsNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<T> for ConsNode<T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T: Debug> Debug for ConsBranch<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Some(node) => write!(f, "{:?}", node),
            None => write!(f, "()"),
        }
    }
}

impl<T: Debug> Debug for ConsNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.next.inner {
            Some(node) => write!(f, "({:?}->{:?})", self.value, node),
            None => write!(f, "({:?})", self.value),
        }
    }
}