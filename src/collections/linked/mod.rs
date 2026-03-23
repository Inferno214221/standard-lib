//! Linked collection types. Primarily revolves around [`LinkedList`] and its accompanying
//! [`Cursor`] type.
#![cfg(feature = "linked")]

pub mod cursor;
pub mod list;

#[doc(inline)]
pub use cursor::Cursor;
#[doc(inline)]
pub use list::LinkedList;
