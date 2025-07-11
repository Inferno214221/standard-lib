use std::{hint, marker::PhantomData};

use super::{DoublyLinkedList, ListContents, ListState, Node, NodePtr};
use super::State as PubState;
use super::StateMut as PubStateMut;
use crate::util::option::OptionExtension;

pub struct Cursor<T> {
    pub(crate) state: CursorState<T>,
    pub(crate) _phantom: PhantomData<T>,
}

pub(crate) enum CursorState<T> {
    Empty,
    Full(CursorContents<T>),
}

pub(crate) struct CursorContents<T> {
    pub list: ListContents<T>,
    pub pos: CursorPosition<T>,
}

use CursorState::*;

pub(crate) enum CursorPosition<T> {
    Head,
    Tail,
    Ptr(NodePtr<T>),
}

use CursorPosition::*;

impl<T> Cursor<T> {
    pub const fn list(self) -> DoublyLinkedList<T> {
        match self.state {
            Empty => DoublyLinkedList {
                state: ListState::Empty,
                _phantom: PhantomData,
            },
            Full(CursorContents { list, .. }) => DoublyLinkedList {
                state: ListState::Full(list),
                _phantom: PhantomData,
            },
        }
    }

    pub const fn read(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { pos, .. }) => match pos {
                Ptr(node) => Some(node.value()),
                _ => None,
            },
        }
    }

    pub const fn read_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { pos, .. }) => match pos {
                Ptr(node) => Some(node.value_mut()),
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
                Ptr(node) => match node.next() {
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
                Ptr(node) => match node.next_mut() {
                    Some(next_node) => Some(next_node.value_mut()),
                    None => None,
                },
            },
        }
    }

    pub const fn move_next(&mut self) -> &mut Self {
        match &mut self.state {
            Empty => (),
            Full(CursorContents { list, pos }) => match pos {
                Head => *pos = Ptr(list.head),
                Tail => (),
                Ptr(node) => {
                    match node.next() {
                        Some(next_node) => *pos = Ptr(*next_node),
                        None => *pos = Tail,
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
                Ptr(ptr) => {
                    list.len = list.len.checked_add(1).expect("Capacity overflow!");

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
                                list.len = unsafe { list.len.checked_sub(1).unreachable() };
                            },
                            None => self.state = Empty,
                        }
                        Some(node.value)
                    },
                    Tail => None,
                    Ptr(ptr) => {
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
                                list.len = unsafe { list.len.checked_sub(1).unreachable() };
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

    // TODO: prev methods
    
    pub const fn is_head(&self) -> bool {
        match &self.state {
            Empty => false,
            Full(CursorContents { pos, .. }) => matches!(pos, Head),
        }
    }

    pub const fn is_tail(&self) -> bool {
        match &self.state {
            Empty => false,
            Full(CursorContents { pos, .. }) => matches!(pos, Tail),
        }
    }

    pub const fn state<'a>(&'a self) -> PubState<'a, T> {
        match &self.state {
            Empty => PubState::Empty,
            Full(CursorContents { pos, .. }) => match pos {
                Head => PubState::Head,
                Tail => PubState::Tail,
                Ptr(ptr) => PubState::Node(ptr.value()),
            },
        }
    }

    pub const fn state_mut<'a>(&'a mut self) -> PubStateMut<'a, T> {
        match &mut self.state {
            Empty => PubStateMut::Empty,
            Full(CursorContents { pos, .. }) => match pos {
                Head => PubStateMut::Head,
                Tail => PubStateMut::Tail,
                Ptr(ptr) => PubStateMut::Node(ptr.value_mut()),
            },
        }
    }

    pub fn get_rel(&self, offset: isize) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match offset.signum() {
                0 => match pos {
                    Ptr(ptr) => Some(ptr.value()),
                    _ => None,
                },
                -1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.tail, offset.abs() - 1),
                        Ptr(ptr) => (*ptr, offset.abs()),
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
                        Ptr(ptr) => (*ptr, offset),
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

    pub fn get_rel_mut(&mut self, offset: isize) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full(CursorContents { list, pos }) => match offset.signum() {
                0 => match pos {
                    Ptr(ptr) => Some(ptr.value_mut()),
                    _ => None,
                },
                -1 => {
                    let (mut ptr, mut steps) = match pos {
                        Head => return None,
                        Tail => (list.tail, offset.abs() - 1),
                        Ptr(ptr) => (*ptr, offset.abs()),
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
                        Ptr(ptr) => (*ptr, offset),
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
}

// Wrapping DoublyLinkedList.
impl<T> Cursor<T> {
    pub const fn len(&self) -> usize {
        match &self.state {
            Empty => 0,
            Full(CursorContents { list, .. }) => list.len.get(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.state, Empty)
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
                    Ptr(node) if *node == list.head => *pos = Head,
                    _ => (),
                }
                
                let node = list.head.take_node();

                match list.len.checked_sub(1) {
                    Some(new_len) => {
                        // SAFETY: Previous length is greater than 1, so the last element is
                        // preceded by at least one more.
                        let new_head = unsafe { node.next.unreachable() };
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

    // TODO: pub fn pop_back(&mut self) -> Option<T>

    // TODO: Should this return an Option<&T>?
    pub fn get(&self, index: usize) -> &T {
        self.checked_seek(index).value()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.checked_seek(index).value_mut()
    }

    // TODO: more redirection to list functions
}

impl<T> Cursor<T> {
    pub(crate) fn checked_seek(&self, index: usize) -> NodePtr<T> {
        self.checked_contents_for_index(index).seek(index)
    }

    pub(crate) fn checked_contents_for_index(&self, index: usize) -> &ListContents<T> {
        match &self.state {
            Empty => panic!("failed to index empty collection"),
            Full(CursorContents { list, .. }) => {
                assert!(
                    index < list.len.get(),
                    "index {} out of bounds for collection with {} elements",
                    index,
                    list.len.get()
                );
                list
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

// impl<T: Debug> Debug for Cursor<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Cursor")
//             // .field("list", &self.list)
//             .field("curr", &self.curr.value())
//             .finish()
//     }
// }
