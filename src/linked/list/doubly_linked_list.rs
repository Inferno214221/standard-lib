use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::num::NonZero;

use super::{Cursor, NodeRef, Node, Iter, IterMut};

pub struct DoublyLinkedList<T> {
    pub(crate) state: ListState<T>,
    pub(crate) _phantom: PhantomData<T>
}

pub(crate) enum ListState<T> {
    Empty,
    // Single(NodeRef<T>),
    Full(Inner<T>)
}

pub(crate) struct Inner<T> {
    pub len: Length,
    pub head: NodeRef<T>,
    pub tail: NodeRef<T>
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Length(NonZero<usize>);

impl Length {
    pub const fn checked_add(self, other: usize) -> Option<Length> {
        Length::wrap_non_zero(self.0.checked_add(other))
    }

    pub const fn checked_sub(self, other: usize) -> Option<Length> {
        Length::wrap_non_zero(match self.0.get().checked_sub(other) {
            Some(res) => NonZero::new(res),
            None => None,
        })
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }

    pub const fn wrap_non_zero(value: Option<NonZero<usize>>) -> Option<Length> {
        match value {
            Some(res) => Some(Length(res)),
            None => None,
        }
    }
}

const ONE: Length = Length(NonZero::<usize>::MIN);

use ListState::*;

impl<T> DoublyLinkedList<T> {
    pub const fn new() -> DoublyLinkedList<T> {
        DoublyLinkedList {
            state: Empty,
            _phantom: PhantomData
        }
    }

    pub const fn len(&self) -> usize {
        match self.state {
            Empty => 0,
            Full(Inner { len, .. }) => len.get(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn front(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(Inner { head, .. }) => Some(head.value()),
        }
    }

    pub const fn front_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(Inner { mut head, .. }) => Some(head.value_mut()),
        }
    }

    pub const fn back(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(Inner { tail, .. }) => Some(tail.value()),
        }
    }

    pub const fn back_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(Inner { mut tail, .. }) => Some(tail.value_mut()),
        }
    }

    pub fn push_front(&mut self, value: T) {
        let new_node = NodeRef::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        match self.state {
            Empty => {
                self.state = Full(Inner {
                    len: ONE,
                    head: new_node,
                    tail: new_node,
                });
            },
            Full(Inner { ref mut len, ref mut head, .. }) => {
                *head.prev_mut() = Some(new_node);
                *new_node.next_mut() = Some(*head);
                *head = new_node;
                *len = len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn push_back(&mut self, value: T) {
        let new_node = NodeRef::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        match self.state {
            Empty => {
                self.state = Full(Inner {
                    len: ONE,
                    head: new_node,
                    tail: new_node,
                });
            },
            Full(Inner { ref mut len, ref mut tail, .. }) => {
                *tail.prev_mut() = Some(new_node);
                *new_node.next_mut() = Some(*tail);
                *tail = new_node;
                *len = len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.state {
            Empty => None,
            Full(Inner { ref mut len, ref mut head, .. }) => {
                let node = head.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // UNWRAP: Previous length is greater than 1, so the first element is
                        // followed by at least one more.
                        let new_head = node.next.unwrap();
                        *head = new_head;
                        *new_head.prev_mut() = None;
                        *len = new_len;
                    },
                    None => {
                        self.state = Empty;
                    },
                }

                Some(node.value)
            },
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        match self.state {
            Empty => None,
            Full(Inner { len, tail, .. }) if len == ONE => {
                let node = tail.take_node();

                self.state = Empty;

                Some(node.value)
            },
            Full(Inner { ref mut len, ref mut tail, .. }) => {
                let node = tail.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // UNWRAP: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = node.prev.unwrap();
                        *tail = new_tail;
                        *new_tail.next_mut() = None;
                        *len = new_len;
                    },
                    None => {
                        self.state = Empty;
                    },
                }

                Some(node.value)
            },
        }
    }

    pub fn get(&self, index: usize) -> &T {
        self.checked_seek(index)
            .value()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.checked_seek(index)
            .value_mut()
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let inner = self.checked_inner_for_index_mut(index - 1);
        match index {
            0 => {
                self.push_front(value)
            },
            val if val == inner.len.get() => {
                self.push_back(value)
            },
            val => {
                let prev_node = inner.seek(val - 1);

                let node = NodeRef::from_node(Node {
                    value,
                    prev: Some(prev_node),
                    next: *prev_node.next(),
                });

                // UNWRAP: For this branch, we aren't adding at the front or back, so the node
                // before the given index has a next node.
                *prev_node.next().unwrap().prev_mut() = Some(node);
                *prev_node.next_mut() = Some(node);

                inner.len = inner.len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        let inner = self.checked_inner_for_index_mut(index);
        match index {
            0 => {
                // UNWRAP: Inner is already check to be valid for the provided index.
                self.pop_front().unwrap()
            },
            val if val == inner.len.get() - 1 => {
                // UNWRAP: Inner is already check to be valid for the provided index.
                self.pop_back().unwrap()
            },
            val => {
                let node = inner.seek(val).take_node();

                // UNWRAP: For this branch, both prev and next must be defined. Head and tail
                // versions are handled with pop front / back branches.
                *node.prev.unwrap().next_mut() = node.next;
                *node.next.unwrap().prev_mut() = node.prev;
                // UNWRAP: If the length was 1, we would have matched one of the previous branches.
                inner.len = inner.len.checked_sub(1).unwrap();

                node.value
            },
        }
    }

    pub fn replace(&mut self, index: usize, new_value: T) -> T {
        mem::replace(
            self.checked_seek(index).value_mut(),
            new_value
        )
    }

    pub fn append(&mut self, other: DoublyLinkedList<T>) {
        match self.state {
            Empty => *self = other,
            Full(ref mut self_inner) => match other.state {
                Empty => {},
                Full(ref other_inner) => {
                    *self_inner.tail.next_mut() = Some(other_inner.head);
                    *other_inner.head.prev_mut() = Some(self_inner.tail);
                    self_inner.tail = other_inner.tail;

                    // TODO: proper handling
                    self_inner.len = self_inner.len.checked_add(other_inner.len.get()).unwrap();
                },
            },
        }
    }

    pub fn cursor_front(mut self) -> Option<Cursor<T>> {
        match mem::replace(&mut self.state, Empty) {
            Empty => None,
            Full(inner) => {
                Some(
                    Cursor {
                        curr: inner.head,
                        list: inner,
                    }
                )
            },
        }
    }

    pub fn cursor_back(mut self) -> Option<Cursor<T>> {
        match mem::replace(&mut self.state, Empty) {
            Empty => None,
            Full(inner) => {
                Some(
                    Cursor {
                        curr: inner.tail,
                        list: inner,
                    }
                )
            },
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }
}

impl<T> DoublyLinkedList<T> {
    pub(crate) fn checked_seek(&self, index: usize) -> NodeRef<T> {
        self.checked_inner_for_index(index).seek(index)
    }

    pub(crate) fn checked_inner_for_index(&self, index: usize) -> &Inner<T> {
        match self.state {
            Empty => panic!("failed to index empty collection"),
            Full(ref inner) => {
                assert!(
                    index < inner.len.get(),
                    "index {} out of bounds for collection with {} elements",
                    index, inner.len.get()
                );
                inner
            },
        }
    }

    pub(crate) fn checked_inner_for_index_mut(&mut self, index: usize) -> &mut Inner<T> {
        match self.state {
            Empty => panic!("failed to index empty collection"),
            Full(ref mut inner) => {
                assert!(
                    index < inner.len.get(),
                    "index {} out of bounds for collection with {} elements",
                    index, inner.len.get()
                );
                inner
            },
        }
    }

    pub(crate) fn verify_double_links(&self) {
        match self.state {
            Empty => {},
            Full(Inner { head, tail, .. }) => {
                let mut curr = head;
                while let Some(next) = curr.next() {
                    assert!(next.prev().unwrap() == curr);
                    curr = *next;
                }
                assert!(tail == curr);
            },
        }
    }
}

impl<T> Inner<T> {
    pub(crate) fn seek(&self, index: usize) -> NodeRef<T> {
        if index < self.len.get() / 2 {
            self.seek_fwd(index)
        } else {
            self.seek_bwd(index)
        }
    }

    pub(crate) fn seek_fwd(&self, index: usize) -> NodeRef<T> {
        let mut curr = self.head;
        for _ in 0..index {
            curr = curr.next().unwrap();
        }
        curr
    }

    pub(crate) fn seek_bwd(&self, index: usize) -> NodeRef<T> {
        let mut curr = self.tail;
        let upper = self.len.checked_sub(index).unwrap().get();
        for _ in 1..upper {
            curr = curr.prev().unwrap();
        }
        curr
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        match self.state {
            Empty => {},
            Full(Inner { head, .. }) => {
                let mut curr = head.take_node();
                loop {
                    drop(curr.value);
                    match curr.next {
                        Some(next) => curr = next.take_node(),
                        None => break,
                    }
                }
            },
        }
    }
}

// impl Extend for DLinkedList

impl<T: Debug> Debug for DoublyLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DLinkedList")
            .field("len", &self.len())
            .field(
                "elements",
                &(self.iter().map(
                    |v| format!("({v:?}) -> ")
                ).collect::<String>() + "End")
            )
            .finish()
    }
}