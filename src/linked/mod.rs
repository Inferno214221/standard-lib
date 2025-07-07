//! Linked collection types. Primarily revolves around [`DoublyLinkedList`] and its accompanying
//! [`Cursor`] type.

pub mod list;

#[doc(inline)]
pub use list::{Cursor, DoublyLinkedList};
