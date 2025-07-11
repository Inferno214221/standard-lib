use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};
use std::ptr;

use super::{Iter, IterMut, Length, Node, NodePtr, ONE};
use crate::linked::cursor::{Cursor, CursorContents, CursorPosition, CursorState};
use crate::contiguous::Vector;
use crate::util::option::OptionExtension;
use crate::util::result::ResultExtension;
#[doc(inline)]
pub use crate::util::error::IndexOutOfBounds;

/// A list with links in both directions. See also: [`Cursor`] for bi-directional iteration and
/// traversal.
///
/// # Time Complexity
/// For this analysis of time complexity, variables are defined as follows:
/// - `n`: The number of items in the LinkedList.
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
/// applications unless LinkedList and the accompanying [`Cursor`] type's `O(1)` methods are being
/// heavily utilized.
pub struct LinkedList<T> {
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

impl<T> LinkedList<T> {
    pub const fn new() -> LinkedList<T> {
        LinkedList {
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
        match &self.state {
            Empty => true,
            Full { .. } => false,
        }
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
        match &mut self.state {
            Empty => self.state = ListState::single(value),
            Full(contents) => contents.push_front(value),
        }
    }

    pub fn push_back(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = ListState::single(value),
            Full(contents) => contents.push_back(value),
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, head, .. }) => {
                let node = head.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the first element is
                        // preceded by at least one more.
                        let new_head = unsafe { node.next.unreachable() };
                        *head = new_head;
                        *new_head.prev_mut() = None;
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(node.value)
            },
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, tail, .. }) => {
                let node = tail.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = unsafe { node.prev.unreachable() };
                        *tail = new_tail;
                        *new_tail.next_mut() = None;
                        *len = new_len;
                    },
                    None => self.state = Empty,
                }

                Some(node.value)
            },
        }
    }

    pub fn get(&self, index: usize) -> &T {
        self.checked_seek(index).throw().value()
    }

    pub fn try_get(&self, index: usize) -> Option<&T> {
        Some(self.checked_seek(index).ok()?.value())
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.checked_seek(index).throw().value_mut()
    }

    pub fn try_get_mut(&mut self, index: usize) -> Option<&mut T> {
        Some(self.checked_seek(index).ok()?.value_mut())
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let contents = self.checked_contents_for_index_mut(index - 1).throw();
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

    // TODO: pub fn try_insert(&mut self, index: usize, value: T) -> ?

    pub fn remove(&mut self, index: usize) -> T {
        let contents = self.checked_contents_for_index_mut(index).throw();
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

    // TODO: pub fn try_remove(&mut self, index: usize) -> Option<T>

    pub fn replace(&mut self, index: usize, new_value: T) -> T {
        mem::replace(
            self.checked_seek(index).throw().value_mut(),
            new_value
        )
    }

    // TODO: pub fn try_replace(&mut self, index: usize, new_value: T) -> Option<T>

    pub fn append(&mut self, other: LinkedList<T>) {
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
                Full(contents) => CursorState::Full(CursorContents {
                    pos: CursorPosition::Head, // TODO: Should the cursor start before head?
                    list: contents,
                }),
            },
            _phantom: PhantomData,
        }
    }

    pub fn cursor_back(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full(CursorContents {
                    pos: CursorPosition::Tail, // TODO: Should the cursor start after tail?
                    list: contents,
                }),
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

impl<T> LinkedList<T> {
    pub(crate) fn checked_seek(&self, index: usize) -> Result<NodePtr<T>, IndexOutOfBounds> {
        Ok(self.checked_contents_for_index(index)?.seek(index))
    }

    pub(crate) const fn checked_contents_for_index(
        &self,
        index: usize,
    ) -> Result<&ListContents<T>, IndexOutOfBounds> {
        match &self.state {
            Empty => Err(IndexOutOfBounds { index, len: 0 }),
            Full(contents) => {
                let len = contents.len.get();
                if index < len {
                    Ok(contents)
                } else {
                    Err(IndexOutOfBounds { index, len })
                }
            },
        }
    }

    pub(crate) const fn checked_contents_for_index_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut ListContents<T>, IndexOutOfBounds> {
        match &mut self.state {
            Empty => Err(IndexOutOfBounds { index, len: 0 }),
            Full(contents) => {
                let len = contents.len.get();
                if index < len {
                    Ok(contents)
                } else {
                    Err(IndexOutOfBounds { index, len })
                }
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

    pub(crate) fn push_front(&mut self, value: T) {
        self.len = self.len.checked_add(1).expect("Capacity overflow!");

        let node = NodePtr::from_node(Node {
            value,
            prev: None,
            next: Some(self.head),
        });

        *self.head.prev_mut() = Some(node);
        self.head = node;
    }

    pub(crate) fn push_back(&mut self, value: T) {
        self.len = self.len.checked_add(1).expect("Capacity overflow!");

        let node = NodePtr::from_node(Node {
            value,
            prev: Some(self.tail),
            next: None,
        });

        *self.tail.next_mut() = Some(node);
        self.tail = node;
    }

    pub(crate) fn wrap_one(value: T) -> ListContents<T> {
        let node = NodePtr::from_node(Node {
            value,
            prev: None,
            next: None,
        });

        ListContents {
            len: ONE,
            head: node,
            tail: node,
        }
    }
}

impl<T> ListState<T> {
    pub(crate) fn single(value: T) -> ListState<T> {
        Full(ListContents::wrap_one(value))
    }
}

impl<T> Index<usize> for LinkedList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}


impl<T> IndexMut<usize> for LinkedList<T> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = LinkedList::new();
        for item in iter.into_iter() {
            list.push_back(item);
        }
        list
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for LinkedList<T> {
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

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DLinkedList")
            .field_with("contents", |f| f.debug_list().entries(self.iter()).finish())
            .field("len", &self.len())
            .finish()
    }
}

impl<T: Debug> Display for LinkedList<T> {
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
