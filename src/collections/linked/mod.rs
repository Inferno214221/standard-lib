//! Linked collection types. Primarily revolves around [`LinkedList`] and its accompanying
//! [`Cursor`] type.

pub mod cursor;
pub mod list;

#[doc(inline)]
pub use cursor::Cursor;
#[doc(inline)]
pub use list::LinkedList;
