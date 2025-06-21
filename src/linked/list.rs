use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;

use super::{Link, NodeRef, Node, Iter, IterMut, Cursor};

pub struct DLinkedList<T> {
    pub(crate) head: Link<T>,
    pub(crate) tail: Link<T>,
    pub(crate) len: usize,
    _phantom: PhantomData<T>
}

impl<T> DLinkedList<T> {
    pub fn new() -> DLinkedList<T> {
        DLinkedList {
            head: None,
            tail: None,
            len: 0,
            _phantom: PhantomData
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| node.value())
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| node.value_mut())
    }

    pub fn back(&self) -> Option<&T> {
        self.tail.as_ref().map(|node| node.value())
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.as_mut().map(|node| node.value_mut())
    }

    pub fn push_back(&mut self, value: T) {
        let new_node = NodeRef::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        if let Some(old_tail) = self.tail {
            *old_tail.next_mut() = Some(new_node);
            *new_node.prev_mut() = Some(old_tail);
            self.tail = Some(new_node);
        } else {
            self.head = Some(new_node);
            self.tail = Some(new_node);
        }
        self.len += 1;
    }

    pub fn push_front(&mut self, value: T) {
        let new_node = NodeRef::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        if let Some(old_head) = self.head {
            *old_head.prev_mut() = Some(new_node);
            *new_node.next_mut() = Some(old_head);
            self.head = Some(new_node);
        } else {
            self.head = Some(new_node);
            self.tail = Some(new_node);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        match self.tail {
            Some(old_tail) if self.len == 1 => {
                debug_assert!(self.head == self.tail);

                let node = old_tail.take_node();

                self.head = None;
                self.tail = None;
                self.len = 0;

                Some(node.value)
            },
            Some(old_tail) => {
                debug_assert!(self.head != self.tail);
                debug_assert!(self.len > 1);

                let node = old_tail.take_node();

                let new_tail = node.prev.unwrap();
                self.tail = Some(new_tail);
                *new_tail.next_mut() = None;
                self.len -= 1;

                Some(node.value)
            },
            _ => {
                debug_assert!(self.head == None);
                debug_assert!(self.len == 0);

                None
            },
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.head {
            Some(old_head) if self.len == 1 => {
                debug_assert!(self.head == self.tail);

                let node = old_head.take_node();

                self.head = None;
                self.tail = None;
                self.len = 0;

                Some(node.value)
            },
            Some(old_head) => {
                debug_assert!(self.head != self.tail);
                debug_assert!(self.len > 1);

                let node = old_head.take_node();

                let new_head = node.prev.unwrap();
                self.head = Some(new_head);
                *new_head.prev_mut() = None;
                self.len -= 1;

                Some(node.value)
            },
            _ => {
                debug_assert!(self.head == None);
                debug_assert!(self.len == 0);

                None
            },
        }
    }

    pub fn get(&self, index: usize) -> &T {
        self.check_index(index);

        unsafe { &self.seek(index).as_non_null().as_ref().value }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.check_index(index);

        unsafe { &mut self.seek(index).as_non_null().as_mut().value }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    pub fn into_cursor(self) -> Option<Cursor<T>> {
        self.head.map(|head| Cursor {
            list: self,
            curr: head
        })
    }
}

impl<T> DLinkedList<T> {
    pub(crate) fn check_index(&self, index: usize) {
        assert!(
            index <= self.len,
            "Index {} out of bounds for DLinkedList with len {}", index, self.len
        );
    }

    pub(crate) fn seek(&self, index: usize) -> NodeRef<T> {
        if index < self.len / 2 {
            self.seek_fwd(index)
        } else {
            self.seek_bwd(index)
        }
    }

    pub(crate) fn seek_fwd(&self, index: usize) -> NodeRef<T> {
        // TODO: can't call unwrap without handling the possibility of len = 0.
        let mut curr = self.head.unwrap();
        for _ in 0..index {
            curr = curr.next().unwrap();
        }
        curr
    }

    pub(crate) fn seek_bwd(&self, index: usize) -> NodeRef<T> {
        let mut curr = self.tail.unwrap();
        for _ in 1..(self.len - index) {
            curr = curr.prev().unwrap();
        }
        curr
    }
}

impl<T> Default for DLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for DLinkedList<T> {
    fn drop(&mut self) {
        self.head.map(|head| {
            let mut curr = head.take_node();
            loop {
                drop(curr.value);
                match curr.next {
                    Some(next) => curr = next.take_node(),
                    None => break,
                }
            }
        });
    }
}

// impl Extend for DLinkedList

impl<T: Debug> Debug for DLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DLinkedList")
            .field("len", &self.len)
            .field(
                "elements",
                &(self.iter().map(
                    |v| format!("({v:?}) -> ")
                ).collect::<String>() + "End")
            )
            .finish()
    }
}