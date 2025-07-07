use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;

use crate::linked::list::{DoublyLinkedList, Link, ListState, Node, NodeRef};

pub struct Cursor<T> {
    pub(crate) list: DoublyLinkedList<T>,
    pub(crate) curr: Link<T>
}

impl<T> Cursor<T> {
    pub fn list(self) -> DoublyLinkedList<T> {
        self.list
    }
    // TODO: essentially re-write Cursor with the possibility of an empty list.

    // pub const fn current(&self) -> &T {
    //     self.curr.value()
    // }

    // pub const fn current_mut(&mut self) -> &mut T {
    //     self.curr.value_mut()
    // }

    // pub fn peek_next(&self) -> Option<&T> {
    //     self.curr.next().as_ref().map(|next| next.value())
    // }

    // pub fn peek_next_mut(&mut self) -> Option<&mut T> {
    //     self.curr.next_mut().as_mut().map(|next| next.value_mut())
    // }

    // // TODO: should this return self??
    // pub fn move_next(&mut self) -> Option<&mut Self> {
    //     match self.curr.next() {
    //         Some(next) => {
    //             self.curr = *next;
    //             Some(self)
    //         },
    //         None => None,
    //     }
    // }

    // pub fn peek_prev(&self) -> Option<&T> {
    //     self.curr.prev().as_ref().map(|prev| prev.value())
    // }

    // pub fn peek_prev_mut(&mut self) -> Option<&mut T> {
    //     self.curr.prev_mut().as_mut().map(|prev| prev.value_mut())
    // }

    // // TODO: should this return self??
    // pub fn move_prev(&mut self) -> Option<&mut Self> {
    //     match self.curr.prev() {
    //         Some(prev) => {
    //             self.curr = *prev;
    //             Some(self)
    //         },
    //         None => None,
    //     }
    // }

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