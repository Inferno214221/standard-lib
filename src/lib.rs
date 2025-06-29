#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]
#![feature(trusted_len)]

// #![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_panics_doc)]

#![allow(clippy::module_inception)]

/// Contiguous collection types. Namely [`Array`](contiguous::Array) and
/// [`Vector`](contiguous::Vector) for contiguous data storage that varies in size at runtime.
pub mod contiguous;

/// Linked collection types. Primarily revolves around
/// [`DoublyLinkedList`](linked::DoublyLinkedList) and its accompanying [`Cursor`](linked::Cursor)
/// type.
pub mod linked;

pub mod hash;