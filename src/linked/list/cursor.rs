use std::marker::PhantomData;

use super::{DoublyLinkedList, ListContents, ListState, Node, NodePtr, ONE};
use crate::util::option::OptionExtension;

pub struct Cursor<T> {
    pub(crate) state: CursorState<T>,
    pub(crate) _phantom: PhantomData<T>,
}

pub(crate) enum CursorState<T> {
    Empty,
    Full {
        list: ListContents<T>,
        pos: CursorPosition<T>,
    },
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
            Full { list, .. } => DoublyLinkedList {
                state: ListState::Full(list),
                _phantom: PhantomData,
            },
        }
    }

    pub const fn read(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full { pos, .. } => match pos {
                Head => None,
                Tail => None,
                Ptr(node) => Some(node.value()),
            },
        }
    }

    pub const fn read_mut(&mut self) -> Option<&mut T> {
        match &mut self.state {
            Empty => None,
            Full { pos, .. } => match pos {
                Head => None,
                Tail => None,
                Ptr(node) => Some(node.value_mut()),
            },
        }
    }

    pub const fn read_next(&self) -> Option<&T> {
        match &self.state {
            Empty => None,
            Full { list, pos } => match pos {
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
            Full { list, pos } => match pos {
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
            Full { list, pos } => match pos {
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
            Empty => {
                let node = NodePtr::from_node(Node {
                    value,
                    prev: None,
                    next: None,
                });

                self.state = Full {
                    list: ListContents {
                        len: ONE,
                        head: node,
                        tail: node,
                    },
                    pos: Head,
                };
            },
            Full { list, pos } => {
                list.len = list.len.checked_add(1)
                    .expect("Capacity overflow!");
                
                match pos {
                    Head => {
                        let node = NodePtr::from_node(Node {
                            value,
                            prev: None,
                            next: Some(list.head),
                        });

                        *list.head.prev_mut() = Some(node);
                        list.head = node;
                    },
                    Tail => {
                        let node = NodePtr::from_node(Node {
                            value,
                            prev: Some(list.tail),
                            next: None,
                        });

                        *list.tail.next_mut() = Some(node);
                        list.tail = node;
                    },
                    Ptr(ptr) => {
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
                }
            },
        }
    }

    pub fn pop_next(&mut self) -> Option<T> {
        match &mut self.state {
            Empty => None,
            Full { list, pos } => {
                match pos {
                    Head => {
                        let node = list.head.take_node();
                        match node.next {
                            Some(next_node) => {
                                *next_node.prev_mut() = None;
                                list.head = next_node;
                                // UNREACHABLE: We've removed 1 node from a list we know to have at
                                // least two: node and next_node.
                                list.len = list.len.checked_sub(1).unreachable();
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
                                // UNREACHABLE: We've removed 1 node from a list we know to have at
                                // least two, pointed to by: ptr and next_ptr.
                                list.len = list.len.checked_sub(1).unreachable();
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

    // pub fn push_next(&mut self, value: T) {
    //     self.list.len = self.list.len.checked_add(1).unwrap(); // TODO: proper handling

    //     let node = NodeRef::from_node(Node {
    //         value,
    //         prev: Some(self.curr),
    //         next: *self.curr.next()
    //     });

    //     match self.curr.next() {
    //         Some(next) => *next.prev_mut() = Some(node),
    //         None => self.list.tail = node,
    //     }

    //     *self.curr.next_mut() = Some(node);
    // }

    // pub fn push_prev(&mut self, value: T) {
    //     self.list.len = self.list.len.checked_add(1).unwrap(); // TODO: proper handling

    //     let node = NodeRef::from_node(Node {
    //         value,
    //         prev: *self.curr.prev(),
    //         next: Some(self.curr)
    //     });

    //     match self.curr.prev() {
    //         Some(prev) => *prev.next_mut() = Some(node),
    //         None => self.list.head = node,
    //     }

    //     *self.curr.prev_mut() = Some(node);
    // }

    // pub fn pop_next(&mut self) -> Option<T> {
    //     self.curr.next_mut().as_mut().map(|next| {
    //         self.list.len = self.list.len.checked_sub(1).unwrap(); // TODO: proper handling
    //         let node = next.take_node();

    //         match node.next {
    //             Some(second_next) => {
    //                 *self.curr.next_mut() = Some(second_next);
    //                 *second_next.prev_mut() = Some(self.curr);
    //             },
    //             None => {
    //                 *self.curr.next_mut() = None;
    //                 self.list.tail = self.curr;
    //             },
    //         }
    //         node.value
    //     })
    // }

    // pub fn pop_prev(&mut self) -> Option<T> {
    //     self.curr.prev_mut().as_mut().map(|prev| {
    //         self.list.len = self.list.len.checked_sub(1).unwrap(); // TODO: proper handling
    //         let node = prev.take_node();

    //         match node.prev {
    //             Some(second_prev) => {
    //                 *self.curr.prev_mut() = Some(second_prev);
    //                 *second_prev.next_mut() = Some(self.curr);
    //             },
    //             None => {
    //                 *self.curr.prev_mut() = None;
    //                 self.list.head = self.curr;
    //             },
    //         }
    //         node.value
    //     })
    // }

    // TODO: redirect most to functions list with some extra handling.
}

// impl<T: Debug> Debug for Cursor<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Cursor")
//             // .field("list", &self.list)
//             .field("curr", &self.curr.value())
//             .finish()
//     }
// }
