use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};

use derive_more::IsVariant;

use super::{Iter, IterMut, Length, Node, NodePtr, ONE};
use crate::collections::contiguous::Vector;
use crate::collections::linked::cursor::{Cursor, CursorContents, CursorPosition, CursorState};
#[doc(inline)]
pub use crate::util::error::{CapacityOverflow, IndexOutOfBounds};
use crate::util::result::ResultExtension;

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
/// | `front/back` | `O(1)` |
/// | `push_front/back` | `O(1)` |
/// | `pop_front/back` | `O(1)` |
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
#[derive(PartialEq, Eq, Hash)]
pub struct LinkedList<T> {
    pub(crate) state: ListState<T>,
    pub(crate) _phantom: PhantomData<T>,
}

#[derive(Default, PartialEq, Eq, Hash, IsVariant)]
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
    /// Creates a new LinkedList with no elements.
    pub const fn new() -> LinkedList<T> {
        LinkedList {
            state: Empty,
            _phantom: PhantomData,
        }
    }

    /// Returns the length of the LinkedList.
    pub const fn len(&self) -> usize {
        self.state.len()
    }

    /// Returns true if the LinkedList contains no elements.
    pub const fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    /// Returns a reference to the first element in the list, if it exists.
    pub const fn front(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(ListContents { head, .. }) => Some(head.value()),
        }
    }

    /// Returns a mutable reference to the first element in the list, if it exists.
    pub const fn front_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(ListContents { mut head, .. }) => Some(head.value_mut()),
        }
    }

    /// Returns a reference to the last element in the list, if it exists.
    pub const fn back(&self) -> Option<&T> {
        match self.state {
            Empty => None,
            Full(ListContents { tail, .. }) => Some(tail.value()),
        }
    }

    /// Returns a mutable reference to the last element in the list, if it exists.
    pub const fn back_mut(&mut self) -> Option<&mut T> {
        match self.state {
            Empty => None,
            Full(ListContents { mut tail, .. }) => Some(tail.value_mut()),
        }
    }

    /// Add the provided element to the front of the LinkedList.
    pub fn push_front(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = ListState::single(value),
            Full(contents) => contents.push_front(value),
        }
    }

    /// Add the provided element to the back of the LinkedList.
    pub fn push_back(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = ListState::single(value),
            Full(contents) => contents.push_back(value),
        }
    }

    /// Removes the first element from the list and returns it, if the list isn't empty.
    pub fn pop_front(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, head, .. }) => {
                let node = head.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the first element is
                        // preceded by at least one more.
                        let new_head = unsafe { node.next.unwrap_unchecked() };
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

    /// Removes the last element from the list and returns it, if the list isn't empty.
    pub fn pop_back(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(ListContents { len, tail, .. }) => {
                let node = tail.take_node();

                match len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = unsafe { node.prev.unwrap_unchecked() };
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

    /// Returns a reference to the element at the provided `index`, panicking on a failure.
    ///
    /// The same functionality can be achieved using the [`Index`] operator.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds of the LinkedList.
    pub fn get(&self, index: usize) -> &T {
        self.try_get(index).throw()
    }

    /// Returns a reference to the element at the provided `index`, returning an [`Err`] on a
    /// failure rather than panicking.
    ///
    /// The same functionality can be achieved using the [`Index`] operator.
    pub fn try_get(&self, index: usize) -> Result<&T, IndexOutOfBounds> {
        Ok(self.checked_seek(index)?.value())
    }

    /// Returns a mutable reference to the element at the provided `index`, panicking on a failure.
    ///
    /// The same functionality can be achieved using the [`IndexMut`] operator.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds of the LinkedList.
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.try_get_mut(index).throw()
    }

    /// Returns a mutable reference to the element at the provided `index`, returning an [`Err`] on
    /// a failure rather than panicking.
    ///
    /// The same functionality can be achieved using the [`IndexMut`] operator.
    pub fn try_get_mut(&mut self, index: usize) -> Result<&mut T, IndexOutOfBounds> {
        Ok(self.checked_seek(index)?.value_mut())
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.try_insert(index, value).throw()
    }

    pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), IndexOutOfBounds> {
        let contents = self.checked_contents_for_index_mut(index - 1)?;
        match index {
            0 => self.push_front(value),
            val if val == contents.len.get() => self.push_back(value),
            val => {
                let prev_node = contents.seek(val - 1);

                contents.len = contents.len.checked_add(1).ok_or(CapacityOverflow).throw();

                let node = NodePtr::from_node(Node {
                    value,
                    prev: Some(prev_node),
                    next: *prev_node.next(),
                });

                // SAFETY: For this branch, we aren't adding at the front or back, so the node
                // before the given index has a next node.
                unsafe { *prev_node.next().unwrap_unchecked().prev_mut() = Some(node); }
                *prev_node.next_mut() = Some(node);
            },
        }
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.try_remove(index).throw()
    }

    pub fn try_remove(&mut self, index: usize) -> Result<T, IndexOutOfBounds> {
        let contents = self.checked_contents_for_index_mut(index).throw();
        match index {
            0 => {
                // SAFETY: contents is already checked to be valid for the provided index.
                Ok(unsafe { self.pop_front().unwrap_unchecked() })
            },
            val if val == contents.last_index() => {
                // SAFETY: contents is already checked to be valid for the provided index.
                Ok(unsafe { self.pop_back().unwrap_unchecked() })
            },
            val => {
                let node = contents.seek(val).take_node();

                // SAFETY: For this branch, both prev and next must be defined. Head and tail
                // versions are handled with pop front / back branches.
                unsafe {
                    *node.prev.unwrap_unchecked().next_mut() = node.next;
                    *node.next.unwrap_unchecked().prev_mut() = node.prev;
                }
                // SAFETY: If the length was 1, we would have matched one of the previous branches.
                contents.len = unsafe { contents.len.checked_sub(1).unwrap_unchecked() };

                Ok(node.value)
            },
        }
    }

    pub fn replace(&mut self, index: usize, new_value: T) -> T {
        self.try_replace(index, new_value).throw()
    }

    pub fn try_replace(&mut self, index: usize, new_value: T) -> Result<T, IndexOutOfBounds> {
        Ok(mem::replace(
            self.checked_seek(index)?.value_mut(),
            new_value,
        ))
    }

    pub fn append(&mut self, other: LinkedList<T>) {
        match &mut self.state {
            Empty => *self = other,
            Full(self_contents) => match &other.state {
                Empty => {},
                Full(other_contents) => {
                    self_contents.len = self_contents.len
                        .checked_add(other_contents.len.get())
                        .ok_or(CapacityOverflow).throw();

                    *self_contents.tail.next_mut() = Some(other_contents.head);
                    *other_contents.head.prev_mut() = Some(self_contents.tail);
                    self_contents.tail = other_contents.tail;
                },
            },
        }
    }

    pub fn cursor_head(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full(CursorContents {
                    pos: CursorPosition::Head,
                    list: contents,
                }),
            },
            _phantom: PhantomData,
        }
    }

    pub fn cursor_tail(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full(CursorContents {
                    pos: CursorPosition::Tail,
                    list: contents,
                }),
            },
            _phantom: PhantomData,
        }
    }

    pub fn cursor_front(mut self) -> Cursor<T> {
        Cursor {
            state: match mem::take(&mut self.state) {
                Empty => CursorState::Empty,
                Full(contents) => CursorState::Full(CursorContents {
                    pos: CursorPosition::Ptr {
                        ptr: contents.head,
                        index: 0,
                    },
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
                    pos: CursorPosition::Ptr {
                        ptr: contents.tail,
                        index: contents.last_index(),
                    },
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

impl<T: Eq> LinkedList<T> {
    pub fn index_of(&self, item: &T) -> Option<usize> {
        for (index, element) in self.iter().enumerate() {
            if element == item { return Some(index); }
        }
        None
    }

    pub fn contains(&self, item: &T) -> bool {
        for i in self.iter() {
            if i == item { return true; }
        }
        false
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

    #[allow(clippy::unwrap_used)]
    pub(crate) fn verify_double_links(&self) {
        match self.state {
            Empty => {},
            Full(ListContents { head, tail, .. }) => {
                let mut curr = head;
                while let Some(next) = curr.next() {
                    // UNWRAP: This needs to panic if prev is None.
                    assert!(next.prev().unwrap() == curr);
                    curr = *next;
                }
                assert!(tail == curr);
            },
        }
    }
}

impl<T> ListContents<T> {
    pub fn seek(&self, index: usize) -> NodePtr<T> {
        if index < self.len.get() / 2 {
            self.seek_fwd(index, self.head)
        } else {
            self.seek_bwd(self.last_index() - index, self.tail)
        }
    }

    pub fn seek_fwd(&self, count: usize, mut node: NodePtr<T>) -> NodePtr<T> {
        for _ in 0..count {
            node = node.next().unwrap();
        }
        node
    }

    pub fn seek_bwd(&self, count: usize, mut node: NodePtr<T>) -> NodePtr<T> {
        for _ in 0..count {
            node = node.prev().unwrap();
        }
        node
    }

    pub fn push_front(&mut self, value: T) {
        self.len = self.len.checked_add(1).ok_or(CapacityOverflow).throw();

        let node = NodePtr::from_node(Node {
            value,
            prev: None,
            next: Some(self.head),
        });

        *self.head.prev_mut() = Some(node);
        self.head = node;
    }

    pub fn push_back(&mut self, value: T) {
        self.len = self.len.checked_add(1).ok_or(CapacityOverflow).throw();

        let node = NodePtr::from_node(Node {
            value,
            prev: Some(self.tail),
            next: None,
        });

        *self.tail.next_mut() = Some(node);
        self.tail = node;
    }

    pub fn wrap_one(value: T) -> ListContents<T> {
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

    pub const fn last_index(&self) -> usize {
        self.len.get() - 1
    }
}

impl<T> ListState<T> {
    pub fn single(value: T) -> ListState<T> {
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
                    curr = *ptr.next();
                    unsafe { ptr.drop_node(); }
                }
            },
        }
    }
}

impl<T: PartialEq> PartialEq for ListContents<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len { return false; }
        let mut node_a = self.head;
        let mut node_b = other.head;

        loop {
            if node_a.value() != node_b.value() {
                break false;
            }
            match (node_a.next(), node_b.next()) {
                (Some(next_a), Some(next_b)) => {
                    node_a = *next_a;
                    node_b = *next_b;
                },
                // Both sides have the same length, so if they aren't both Some, they are both None.
                // It feels a little neater to do a catchall here then using unreachable_unchecked.
                _ => break true,
            }
        }
    }
}

impl<T: Eq> Eq for ListContents<T> {}

impl<T: Hash> Hash for ListContents<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        let mut node = self.head;

        loop {
            node.value().hash(state);
            match node.next() {
                Some(next) => {
                    node = *next;
                },
                _ => break,
            }
        }

        // Terminate variable length hashing sequence.
        0xFF.hash(state);
    }
}

impl<T> Clone for ListContents<T> {
    fn clone(&self) -> Self {
        ListContents {
            len: self.len,
            head: self.head,
            tail: self.tail,
        }
    }
}

impl<T> ListState<T> {
    pub const fn len(&self) -> usize {
        match self {
            Empty => 0,
            Full(ListContents { len, .. }) => len.get(),
        }
    }
}

impl<T> Clone for ListState<T> {
    fn clone(&self) -> Self {
        match self {
            Empty => Empty,
            Full(contents) => Full(contents.clone()),
        }
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkedList")
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
