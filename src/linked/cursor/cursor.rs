use std::hash::{Hash, Hasher};
use std::hint;
use std::marker::PhantomData;

use super::{State, StateMut};
use crate::linked::list::{Length, LinkedList, ListContents, ListState, Node, NodePtr};
use crate::util::error::{CapacityOverflow, IndexOutOfBounds};
use crate::util::result::ResultExtension;

use derive_more::IsVariant;

/// A type for bi-directional traversal and mutation of [`LinkedList`]s. See
/// [`LinkedList::cursor_front`] and [`LinkedList::cursor_back`] to create one.
#[derive(Hash)]
pub struct Cursor<T> {
    pub(crate) state: CursorState<T>,
    pub(crate) _phantom: PhantomData<T>,
}

#[derive(Hash, IsVariant)]
pub(crate) enum CursorState<T> {
    Empty,
    Full(CursorContents<T>),
}

#[derive(Hash)]
pub(crate) struct CursorContents<T> {
    pub list: ListContents<T>,
    pub pos: CursorPosition<T>,
}

use CursorState::*;


#[derive(IsVariant)]
pub(crate) enum CursorPosition<T> {
    Head,
    Tail,
    Ptr {
        ptr: NodePtr<T>,
        index: usize,
    },
}

use CursorPosition::*;

impl<T> Cursor<T> {
    pub const fn list(self) -> LinkedList<T> {
        match self.state {
            Empty => LinkedList {
                state: ListState::Empty,
                _phantom: PhantomData,
            },
            Full(CursorContents { list, .. }) => LinkedList {
                state: ListState::Full(list),
                _phantom: PhantomData,
            },
        }
    }

    pub const fn index(&self) -> Option<usize> {
        match &self.state {
            Empty => None,
            Full(CursorContents { pos, .. }) => match pos {
                Head | Tail => None,
                Ptr { index, .. } => Some(*index),
            },
        }
    }

    pub const fn read(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { pos, .. }) => match pos {
                Ptr { ptr, .. } => Some(ptr.value()),
                _ => None,
            },
        }
    }

    pub const fn read_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { pos, .. }) => match pos {
                Ptr { ptr, .. } => Some(ptr.value_mut()),
                _ => None,
            },
        }
    }

    pub const fn read_next(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match pos {
                Head => Some(list.head.value()),
                Tail => None,
                Ptr { ptr, .. } => match ptr.next() {
                    Some(next_node) => Some(next_node.value()),
                    None => None,
                },
            },
        }
    }

    pub const fn read_next_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match pos {
                Head => Some(list.head.value_mut()),
                Tail => None,
                Ptr { ptr, .. } => match ptr.next_mut() {
                    Some(next_node) => Some(next_node.value_mut()),
                    None => None,
                },
            },
        }
    }

    pub const fn read_prev(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match pos {
                Head => None,
                Tail => Some(list.tail.value()),
                Ptr { ptr, .. } => match ptr.prev() {
                    Some(prev_node) => Some(prev_node.value()),
                    None => None,
                },
            },
        }
    }

    pub const fn read_prev_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match pos {
                Head => None,
                Tail => Some(list.tail.value_mut()),
                Ptr { ptr, .. } => match ptr.prev_mut() {
                    Some(prev_node) => Some(prev_node.value_mut()),
                    None => None,
                },
            },
        }
    }

    pub const fn move_next(&mut self) -> &mut Self {
        match &mut self.state {
            Empty => (),
            Full(CursorContents { list, pos }) => match pos {
                Head => *pos = Ptr {
                    ptr: list.head,
                    index: 0,
                },
                Tail => (),
                Ptr { ptr, index } => {
                    match ptr.next() {
                        Some(next_node) => *pos = Ptr {
                            ptr: *next_node,
                            index: *index + 1
                        },
                        None => *pos = Tail,
                    }
                },
            },
        }
        self
    }

    pub const fn move_prev(&mut self) -> &mut Self {
        match &mut self.state {
            Empty => (),
            Full(CursorContents { list, pos }) => match pos {
                Head => (),
                Tail => *pos = Ptr {
                    ptr: list.tail,
                    index: list.last_index(),
                },
                Ptr { ptr, index } => {
                    match ptr.prev() {
                        Some(prev_node) => *pos = Ptr {
                            ptr: *prev_node,
                            index: *index - 1,
                        },
                        None => *pos = Head,
                    }
                },
            },
        }
        self
    }

    pub fn push_next(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = CursorState::single(value, Head),
            Full(CursorContents { list, pos }) => match pos {
                Head => list.push_front(value),
                Tail => list.push_back(value),
                Ptr { ptr, .. } => {
                    list.len = list.len.checked_add(1).ok_or(CapacityOverflow).throw();

                    let node = NodePtr::from_node(Node {
                        value,
                        prev: Some(*ptr),
                        next: *ptr.next(),
                    });

                    match ptr.next_mut() {
                        Some(second_next) => *second_next.prev_mut() = Some(node),
                        None => list.tail = node,
                    }
                    *ptr.next_mut() = Some(node)
                },
            },
        }
    }

    pub fn push_prev(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = CursorState::single(value, Tail),
            Full(CursorContents { list, pos }) => match pos {
                Head => list.push_front(value),
                Tail => list.push_back(value),
                Ptr { ptr, .. } => {
                    list.len = list.len.checked_add(1).ok_or(CapacityOverflow).throw();

                    let node = NodePtr::from_node(Node {
                        value,
                        prev: *ptr.prev(),
                        next: Some(*ptr),
                    });

                    match ptr.prev_mut() {
                        Some(second_prev) => *second_prev.next_mut() = Some(node),
                        None => list.head = node,
                    }
                    *ptr.prev_mut() = Some(node)
                },
            },
        }
    }

    pub fn pop_next(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => {
                match pos {
                    Head => {
                        let node = list.head.take_node();
                        match node.next {
                            Some(next_node) => {
                                *next_node.prev_mut() = None;
                                list.head = next_node;
                                // SAFETY: We've removed 1 node from a list we know to have at least
                                // two: node and next_node.
                                list.len = unsafe { list.len.checked_sub(1).unwrap_unchecked() };
                            },
                            None => self.state = Empty,
                        }
                        Some(node.value)
                    },
                    Tail => None,
                    Ptr { ptr, .. } => {
                        match ptr.next_mut() {
                            Some(next_ptr) => {
                                let next_node = next_ptr.take_node();
                                match next_node.next {
                                    Some(second_next) => {
                                        *second_next.prev_mut() = Some(*ptr);
                                        *ptr.next_mut() = Some(second_next);
                                    },
                                    None => {
                                        list.tail = *ptr;
                                        *ptr.next_mut() = None;
                                    },
                                }
                                // SAFETY: We've removed 1 node from a list we know to have at least
                                // two, pointed to by ptr and next_ptr.
                                list.len = unsafe { list.len.checked_sub(1).unwrap_unchecked() };
                                Some(next_node.value)
                            },
                            // We are on a node without a next value, so we return None, despite not
                            // being empty.
                            None => None,
                        }
                    },
                }
            },
        }
    }

    pub fn pop_prev(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => {
                match pos {
                    Head => None,
                    Tail => {
                        let node = list.tail.take_node();
                        match node.prev {
                            Some(prev_node) => {
                                *prev_node.next_mut() = None;
                                list.head = prev_node;
                                // SAFETY: We've removed 1 node from a list we know to have at least
                                // two: node and next_node.
                                list.len = unsafe { list.len.checked_sub(1).unwrap_unchecked() };
                            },
                            None => self.state = Empty,
                        }
                        Some(node.value)
                    },
                    Ptr { ptr, .. } => {
                        match ptr.prev_mut() {
                            Some(prev_ptr) => {
                                let prev_node = prev_ptr.take_node();
                                match prev_node.prev {
                                    Some(second_prev) => {
                                        *second_prev.next_mut() = Some(*ptr);
                                        *ptr.prev_mut() = Some(second_prev);
                                    },
                                    None => {
                                        list.head = *ptr;
                                        *ptr.prev_mut() = None;
                                    },
                                }
                                // SAFETY: We've removed 1 node from a list we know to have at least
                                // two, pointed to by ptr and prev_ptr.
                                list.len = unsafe { list.len.checked_sub(1).unwrap_unchecked() };
                                Some(prev_node.value)
                            },
                            // We are on a node without a next value, so we return None, despite not
                            // being empty.
                            None => None,
                        }
                    },
                }
            },
        }
    }

    pub const fn state<'a>(&'a self) -> State<'a, T> {
        match &self.state {
            Empty => State::Empty,
            Full(CursorContents { pos, .. }) => match pos {
                Head => State::Head,
                Tail => State::Tail,
                Ptr { ptr, .. } => State::Node(ptr.value()),
            },
        }
    }

    pub const fn state_mut<'a>(&'a mut self) -> StateMut<'a, T> {
        match &mut self.state {
            Empty => StateMut::Empty,
            Full(CursorContents { pos, .. }) => match pos {
                Head => StateMut::Head,
                Tail => StateMut::Tail,
                Ptr { ptr, .. } => StateMut::Node(ptr.value_mut()),
            },
        }
    }
    
    pub const fn is_head(&self) -> bool {
        match &self.state {
            Empty => false,
            Full(CursorContents { pos, .. }) => pos.is_head(),
        }
    }

    pub const fn is_tail(&self) -> bool {
        match &self.state {
            Empty => false,
            Full(CursorContents { pos, .. }) => pos.is_tail(),
        }
    }

    pub fn read_offset(&self, offset: isize) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match offset.signum() {
                0 => match pos {
                    Ptr { ptr, .. } => Some(ptr.value()),
                    _ => None,
                },
                -1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.tail, offset.abs() - 1),
                        Ptr { ptr, .. } => (*ptr, offset.abs()),
                    };
                    
                    while steps > 0 {
                        ptr = (*ptr.prev())?;
                        steps -= 1;
                    }

                    Some(ptr.value())
                },
                1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.head, offset),
                        Ptr { ptr, .. } => (*ptr, offset),
                    };
                    
                    while steps > 0 {
                        ptr = (*ptr.next())?;
                        steps -= 1;
                    }

                    Some(ptr.value())
                },
                // SAFETY: signum returns only one of the options above.
                _ => unsafe { hint::unreachable_unchecked() },
            },
        }
    }

    pub fn read_offset_mut(&mut self, offset: isize) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match offset.signum() {
                0 => match pos {
                    Ptr { ptr, .. } => Some(ptr.value_mut()),
                    _ => None,
                },
                -1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.tail, offset.abs() - 1),
                        Ptr { ptr, .. } => (*ptr, offset.abs()),
                    };
                    
                    while steps > 0 {
                        ptr = (*ptr.prev())?;
                        steps -= 1;
                    }

                    Some(ptr.value_mut())
                },
                1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.head, offset),
                        Ptr { ptr, .. } => (*ptr, offset),
                    };
                    
                    while steps > 0 {
                        ptr = (*ptr.next())?;
                        steps -= 1;
                    }

                    Some(ptr.value_mut())
                },
                // SAFETY: signum returns only one of the options above.
                _ => unsafe { hint::unreachable_unchecked() },
            },
        }
    }

    pub const fn move_offset(&mut self, offset: isize) -> &mut Self {
        match &mut self.state {
            Empty => (),
            Full(CursorContents { list, pos }) => match offset.signum() {
                0 => (),
                -1 => {
                    let (mut ptr, mut steps, mut index) = match pos {
                        Head => return self,
                        Tail => (list.tail, offset.unsigned_abs() - 1, list.last_index()),
                        Ptr { ptr, index } => (*ptr, offset.unsigned_abs(), *index),
                    };
                    index -= steps;
                    
                    while steps > 0 {
                        ptr = match *ptr.prev() {
                            Some(p) => p,
                            None => {
                                *pos = Tail;
                                return self;
                            },
                        };
                        steps -= 1;
                    }

                    *pos = Ptr { ptr, index };
                },
                1 => {
                    let (mut ptr, mut steps, mut index) = match pos {
                        Head => return self,
                        Tail => (list.head, offset as usize, 0),
                        Ptr { ptr, index } => (*ptr, offset as usize, *index),
                    };
                    index += steps;
                    
                    while steps > 0 {
                        ptr = match *ptr.next() {
                            Some(p) => p,
                            None => {
                                *pos = Head;
                                return self;
                            },
                        };
                        steps -= 1;
                    }

                    *pos = Ptr { ptr, index };
                },
                // SAFETY: signum returns only one of the options above.
                _ => unsafe { hint::unreachable_unchecked() },
            },
        }
        self
    }

    pub fn move_to(&mut self, index: usize) -> &mut Self {
        self.try_move_to(index).throw()
    }

    pub fn try_move_to(&mut self, index: usize) -> Result<&mut Self, IndexOutOfBounds> {
        let contents = self.checked_contents_for_index_mut(index)?;
        contents.pos = Ptr { ptr: contents.seek(index), index };
        Ok(self)
    }

    pub const fn split_before(self) -> (LinkedList<T>, LinkedList<T>) {
        match &self.state {
            Empty => (LinkedList::new(), LinkedList::new()),
            Full(CursorContents { list, pos }) => match pos {
                Head => (LinkedList::new(), self.list()),
                Tail => (self.list(), LinkedList::new()),
                Ptr { index, .. }
                    if *index == 0 => (LinkedList::new(), self.list()),
                Ptr { ptr, index } => (
                    LinkedList {
                        state: ListState::Full(ListContents {
                            // SAFETY: index = 0 matches the previous branch.
                            len: unsafe { Length::new_unchecked(*index) },
                            head: list.head,
                            // SAFETY: index != 0, so prev is Some.
                            tail: unsafe { ptr.prev().unwrap_unchecked() },
                        }),
                        _phantom: PhantomData
                    },
                    LinkedList {
                        state: ListState::Full(ListContents {
                            // SAFETY: index is in the range 0..list.len so list.len - index as at
                            // least 1.
                            len: unsafe { Length::new_unchecked(list.len.get() - *index) },
                            head: *ptr,
                            tail: list.tail,
                        }),
                        _phantom: PhantomData
                    },
                ),
            },
        }
    }

    pub const fn split_after(self) -> (LinkedList<T>, LinkedList<T>) {
        match &self.state {
            Empty => (LinkedList::new(), LinkedList::new()),
            Full(CursorContents { list, pos }) => match pos {
                Head => (LinkedList::new(), self.list()),
                Tail => (self.list(), LinkedList::new()),
                Ptr { index, .. }
                    if *index == list.last_index() => (LinkedList::new(), self.list()),
                Ptr { ptr, index } => (
                    LinkedList {
                        state: ListState::Full(ListContents {
                            // SAFETY: value is at least 1.
                            len: unsafe { Length::new_unchecked(*index + 1) },
                            head: list.head,
                            tail: *ptr,
                        }),
                        _phantom: PhantomData
                    },
                    LinkedList {
                        state: ListState::Full(ListContents {
                            // SAFETY: index = list.last_index() matches the previous branch, so
                            // list.last_index() - index is > 0.
                            len: unsafe { Length::new_unchecked(list.last_index() - *index) },
                            // SAFETY: index = list.last_index() matches the previous branch, so
                            // next is Some.
                            head: unsafe { ptr.next().unwrap_unchecked() },
                            tail: list.tail,
                        }),
                        _phantom: PhantomData
                    },
                ),
            },
        }
    }
}

impl<T> Cursor<T> {
    pub const fn len(&self) -> usize {
        match &self.state {
            Empty => 0,
            Full(CursorContents { list, .. }) => list.len.get(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub const fn front(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, .. }) => Some(list.head.value()),
        }
    }

    pub const fn front_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, .. }) => Some(list.head.value_mut()),
        }
    }

    pub const fn back(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, .. }) => Some(list.tail.value()),
        }
    }

    pub const fn back_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, .. }) => Some(list.tail.value_mut()),
        }
    }

    pub fn push_front(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = CursorState::single(value, Head),
            Full(CursorContents { list, .. }) => list.push_front(value),
        }
    }

    pub fn push_back(&mut self, value: T) {
        match &mut self.state {
            Empty => self.state = CursorState::single(value, Head),
            Full(CursorContents { list, .. }) => list.push_back(value),
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => {
                match pos {
                    // If we're pointing to the node we need to pop, move to head. Might be strange
                    // but at least its obvious that we've moved.
                    Ptr { ptr, .. } if *ptr == list.head => *pos = Head,
                    _ => (),
                }
                
                let node = list.head.take_node();

                match list.len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_head = unsafe { node.next.unwrap_unchecked() };
                        list.head = new_head;
                        *new_head.prev_mut() = None;
                        list.len = new_len;
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
            Full(CursorContents { list, pos }) => {
                match pos {
                    Ptr { ptr, .. } if *ptr == list.tail => *pos = Tail,
                    _ => (),
                }
                
                let node = list.tail.take_node();

                match list.len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_tail = unsafe { node.prev.unwrap_unchecked() };
                        list.tail = new_tail;
                        *new_tail.prev_mut() = None;
                        list.len = new_len;
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

    // TODO: more redirection to list functions
}

impl<T> Cursor<T> {
    pub(crate) fn checked_seek(&self, index: usize) -> Result<NodePtr<T>, IndexOutOfBounds> {
        Ok(self.checked_contents_for_index(index)?.seek(index))
    }

    pub(crate) const fn checked_contents_for_index(
        &self,
        index: usize,
    ) -> Result<&CursorContents<T>, IndexOutOfBounds> {
        match &self.state {
            Empty => Err(IndexOutOfBounds { index, len: 0 }),
            Full(contents) => {
                let len = contents.list.len.get();
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
    ) -> Result<&mut CursorContents<T>, IndexOutOfBounds> {
        match &mut self.state {
            Empty => Err(IndexOutOfBounds { index, len: 0 }),
            Full(contents) => {
                let len = contents.list.len.get();
                if index < len {
                    Ok(contents)
                } else {
                    Err(IndexOutOfBounds { index, len })
                }
            },
        }
    }
}

impl<T> CursorState<T> {
    pub(crate) fn single(value: T, pos: CursorPosition<T>) -> CursorState<T>{
        Full(CursorContents {
            list: ListContents::wrap_one(value),
            pos,
        })
    }
}

impl<T> CursorContents<T> {
    pub fn seek(&self, target: usize) -> NodePtr<T> {
        match self.pos {
            Head | Tail => self.list.seek(target),
            Ptr { ptr, index } => {
                let end = if target < self.list.len.get() / 2 {
                    (self.list.head, 0)
                } else {
                    (self.list.tail, self.list.last_index())
                };

                let offset_ptr = target as i128 - index as i128;
                let ptr_dist = offset_ptr.unsigned_abs() as usize;

                let offset_end = target as i128 - end.1 as i128;
                let end_dist = offset_end.unsigned_abs() as usize;
                
                if ptr_dist < end_dist {
                    self.seek_n(ptr_dist, ptr, offset_ptr.is_positive())
                } else {
                    self.seek_n(end_dist, end.0, offset_end.is_positive())
                }
            },
        }
    }

    pub fn seek_n(&self, count: usize, start: NodePtr<T>, forward: bool) -> NodePtr<T> {
        if forward {
            self.list.seek_fwd(count, start)
        } else {
            self.list.seek_bwd(count, start)
        }
    }
}

impl<T> Hash for CursorPosition<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Ptr { index, .. } => index.hash(state),
            other => core::mem::discriminant(other).hash(state)
        }
    }
}

// impl<T: Debug> Debug for Cursor<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Cursor")
//             // .field("list", &self.list)
//             .field("curr", &self.curr.value())
//             .finish()
//     }
// }
