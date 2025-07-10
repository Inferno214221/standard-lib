use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::ptr;

use super::{Cursor, CursorPosition, CursorState, Iter, IterMut, Length, Node, NodePtr, ONE};
use crate::contiguous::Vector;

/// A list with links in both directions. See also: [`Cursor`] for bi-directional iteration and
/// traversal.
///
/// # Time Complexity
/// For this analysis of time complexity, variables are defined as follows:
/// - `n`: The number of items in the DoublyLinkedList.
/// - `i`: The index of the item in question.
///
/// | Method | Complexity |
/// |-|-|
/// | `len` | `O(1)` |
/// | `front` | `O(1)` |
/// | `back` | `O(1)` |
/// | `push_front` | `O(1)` |
/// | `push_back` | `O(1)` |
/// | `pop_front` | `O(1)` |
/// | `pop_back` | `O(1)` |
/// | `get` | `O(min(i, n-i))` |
/// | `insert` | `O(min(i, n-i))` |
/// | `remove` | `O(min(i, n-i))` |
/// | `replace` | `O(min(i, n-i))` |
/// | `append` | `O(1)` |
/// | `contains` | `O(n)` |
///
/// As a general note, modern computer architecture isn't kind to linked lists, (or more
/// importantly, favours contiguous collections) because all `O(i)` or `O(n)` operations will
/// consist primarily of cache misses. For this reason, [`Vector`] should be preferred for most
/// applications unless DoublyLinkedList and the accompanying [`Cursor`] type's `O(1)` methods are
/// being heavily utilized.
pub struct DoublyLinkedList<T> {
    pub(crate) state: ListState<T>,
    pub(crate) _phantom: PhantomData<T>,
}

#[derive(Default)]
pub(crate) enum ListState<T> {
    #[default]
    Empty,
    Full(ListContents<T>),
}

use ListState::*;

pub(crate) struct ListContents<T> {
    pub len: Length,
    pub head: NodePtr<T>,
    pub tail: NodePtr<T>,
}

impl<T> DoublyLinkedList<T> {
    pub const fn new() -> DoublyLinkedList<T> {
        DoublyLinkedList {
            state: Empty,
            _phantom: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        match self.state {
            Empty => 0,
            Full(ListContents { len, .. }) => len.get(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn front(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(ListContents { head, .. }) => Some(head.value()),
        }
    }

    pub const fn front_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(ListContents { mut head, .. }) => Some(head.value_mut()),
        }
    }

    pub const fn back(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(ListContents { tail, .. }) => Some(tail.value()),
        }
    }

    pub const fn back_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(ListContents { mut tail, .. }) => Some(tail.value_mut()),
        }
    }

    pub fn push_front(&mut self, value: T) {
        let new_node = NodePtr::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        match &mut self.state {
            Empty => {
                self.state = Full(ListContents {
                    len: ONE,
                    head: new_node,
                    tail: new_node,
                });
            },
            Full(ListContents { len, head, .. }) => {
                *head.prev_mut() = Some(new_node);
                *new_node.next_mut() = Some(*head);
                *head = new_node;
                *len = len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn push_back(&mut self, value: T) {
        let new_node = NodePtr::from_node(
            Node {
                value,
                prev: None,
                next: None
            }
        );

        match &mut self.state {
            Empty => {
                self.state = Full(ListContents {
                    len: ONE,
                    head: new_node,
                    tail: new_node,
                });
            },
            Full(ListContents { len, tail, .. }) => {
                *tail.next_mut() = Some(new_node);
                *new_node.prev_mut() = Some(*tail);
                *tail = new_node;
                *len = len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, head, .. }) => {
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
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, tail, .. }) if *len == ONE => {
                let node = tail.take_node();

                self.state = Empty;

                Some(node.value)
            },
            Full(ListContents { len, tail, .. }) => {
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
        self.checked_seek(index).value()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.checked_seek(index).value_mut()
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let contents = self.checked_contents_for_index_mut(index - 1);
        match index {
            0 => self.push_front(value),
            val if val == contents.len.get() => self.push_back(value),
            val => {
                let prev_node = contents.seek(val - 1);

                let node = NodePtr::from_node(Node {
                    value,
                    prev: Some(prev_node),
                    next: *prev_node.next(),
                });

                // UNWRAP: For this branch, we aren't adding at the front or back, so the node
                // before the given index has a next node.
                *prev_node.next().unwrap().prev_mut() = Some(node);
                *prev_node.next_mut() = Some(node);

                contents.len = contents.len.checked_add(1).unwrap(); // TODO: proper handling
            },
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        let contents = self.checked_contents_for_index_mut(index);
        match index {
            0 => {
                // UNWRAP: contents is already checked to be valid for the provided index.
                self.pop_front().unwrap()
            },
            val if val == contents.len.get() - 1 => {
                // UNWRAP: contents is already checked to be valid for the provided index.
                self.pop_back().unwrap()
            },
            val => {
                let node = contents.seek(val).take_node();

                // UNWRAP: For this branch, both prev and next must be defined. Head and tail
                // versions are handled with pop front / back branches.
                *node.prev.unwrap().next_mut() = node.next;
                *node.next.unwrap().prev_mut() = node.prev;
                // UNWRAP: If the length was 1, we would have matched one of the previous branches.
                contents.len = contents.len.checked_sub(1).unwrap();

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
        match &mut self.state {
            Empty => *self = other,
            Full(self_contents) => match &other.state {
                Empty => {},
                Full(other_contents) => {
                    *self_contents.tail.next_mut() = Some(other_contents.head);
                    *other_contents.head.prev_mut() = Some(self_contents.tail);
                    self_contents.tail = other_contents.tail;

                    // TODO: proper handling
                    self_contents.len = self_contents.len.checked_add(
                        other_contents.len.get()
                    ).unwrap();
                },
            },
        }
    }

    // TODO: pub fn contains(&self, item: T)

    pub fn cursor_front(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full {
                    pos: CursorPosition::Head, // TODO: Should the cursor start before head?
                    list: contents,
                },
            },
            _phantom: PhantomData,
        }
    }

    pub fn cursor_back(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full {
                    pos: CursorPosition::Tail, // TODO: Should the cursor start after tail?
                    list: contents,
                },
            },
            _phantom: PhantomData,
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
    pub(crate) fn checked_seek(&self, index: usize) -> NodePtr<T> {
        self.checked_contents_for_index(index).seek(index)
    }

    pub(crate) fn checked_contents_for_index(&self, index: usize) -> &ListContents<T> {
        match &self.state {
            Empty => panic!("failed to index empty collection"),
            Full(contents) => {
                assert!(
                    index < contents.len.get(),
                    "index {} out of bounds for collection with {} elements",
                    index,
                    contents.len.get()
                );
                contents
            },
        }
    }

    pub(crate) fn checked_contents_for_index_mut(&mut self, index: usize) -> &mut ListContents<T> {
        match &mut self.state {
            Empty => panic!("failed to index empty collection"),
            Full(contents) => {
                assert!(
                    index < contents.len.get(),
                    "index {} out of bounds for collection with {} elements",
                    index,
                    contents.len.get()
                );
                contents
            },
        }
    }

    pub(crate) fn verify_double_links(&self) {
        match self.state {
            Empty => {},
            Full(ListContents { head, tail, .. }) => {
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

impl<T> ListContents<T> {
    pub(crate) fn seek(&self, index: usize) -> NodePtr<T> {
        if index < self.len.get() / 2 {
            self.seek_fwd(index)
        } else {
            self.seek_bwd(index)
        }
    }

    pub(crate) fn seek_fwd(&self, index: usize) -> NodePtr<T> {
        let mut curr = self.head;
        for _ in 0..index {
            curr = curr.next().unwrap();
        }
        curr
    }

    pub(crate) fn seek_bwd(&self, index: usize) -> NodePtr<T> {
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
            Full(ListContents { head, .. }) => {
                let mut curr = Some(head);
                while let Some(ptr) = curr {
                    unsafe { ptr::drop_in_place(ptr.as_ptr()) };
                    curr = *ptr.next();
                }
            },
        }
    }
}

impl<T> FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = DoublyLinkedList::new();
        for item in iter.into_iter() {
            list.push_back(item);
        }
        list
    }
}

impl<T: Debug> Debug for DoublyLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DLinkedList")
            .field_with("contents", |f| f.debug_list().entries(self.iter()).finish())
            .field("len", &self.len())
            .finish()
    }
}

impl<T: Debug> Display for DoublyLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({})",
            self.iter()
                .map(|i| format!("{i:?}"))
                .collect::<Vector<String>>()
                .join(") -> (")
        )
    }
}
