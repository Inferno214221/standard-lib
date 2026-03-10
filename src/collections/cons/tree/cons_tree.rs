use std::{mem, ops::Deref, rc::Rc};

use super::{Iter, OwnedIter, RcIter, UniqueIter};

#[derive(Debug, Clone)]
pub struct ConsTree<T> {
    pub(crate) inner: Option<Rc<ConsTreeNode<T>>>,
}

#[derive(Debug, Clone)]
pub struct ConsTreeNode<T> {
    pub value: T,
    pub(crate) next: ConsTree<T>,
}

impl<T> ConsTree<T> {
    pub const fn new() -> ConsTree<T> {
        ConsTree {
            inner: None
        }
    }

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

impl<T: Clone> ConsTree<T> {
    pub const fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

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

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.inner.as_deref(),
        }
    }

    pub const fn iter_to_owned(self) -> OwnedIter<T> {
        OwnedIter {
            inner: self,
        }
    }

    pub const fn iter_rc(self) -> RcIter<T> {
        RcIter {
            inner: self,
        }
    }

    pub const fn iter_unique(self) -> UniqueIter<T> {
        UniqueIter {
            inner: self,
        }
    }

    pub fn is_unqiue(&self) -> bool {
        let mut next = &self.inner;
        while let Some(node) = next {
            if !is_unqiue(node) {
                return false;
            }
            next = &node.next.inner;
        }
        true
    }

    pub fn is_head_unqiue(&self) -> bool {
        match &self.inner {
            Some(node) => is_unqiue(node),
            None => true,
        }
    }

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