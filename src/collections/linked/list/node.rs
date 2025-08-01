use std::ptr::NonNull;

pub(crate) type Link<T> = Option<NodePtr<T>>;

// NOTE: This implementation uses Box<T> rather than alloc to allocate space on the heap, because
// Box<T> has the special property that dereferencing it allows a value to be moved out of the heap.

pub(crate) struct Node<T> {
    pub value: T,
    pub prev: Link<T>,
    pub next: Link<T>,
}

#[derive(Debug)]
pub(crate) struct NodePtr<T>(pub NonNull<Node<T>>);

impl<T> NodePtr<T> {
    pub const fn value<'a>(&self) -> &'a T {
        unsafe { &(self.0.as_ref()).value }
    }

    pub const fn value_mut<'a>(&mut self) -> &'a mut T {
        unsafe { &mut (self.0.as_mut()).value }
    }

    pub const fn prev<'a>(&self) -> &'a Link<T> {
        unsafe { &(*self.0.as_ptr()).prev }
    }

    #[allow(clippy::mut_from_ref)]
    pub const fn prev_mut<'a>(&self) -> &'a mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).prev }
    }

    pub const fn next<'a>(&self) -> &'a Link<T> {
        unsafe { &(*self.0.as_ptr()).next }
    }

    #[allow(clippy::mut_from_ref)]
    pub const fn next_mut<'a>(&self) -> &'a mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).next }
    }

    pub fn from_node(node: Node<T>) -> NodePtr<T> {
        NodePtr(Box::into_non_null(Box::new(node)))
    }

    pub fn take_node(self) -> Node<T> {
        unsafe { *Box::from_non_null(self.0) }
    }

    /// Drops the pointed to node by reconstructing the [`Box`] and dropping that. This ensures that
    /// the pointed to memory is deallocated correctly.
    ///
    /// # Safety
    /// The caller mut ensure that any duplicate `NodePtr`s aren't used or explicitly dropped after
    /// calling this function.
    pub unsafe fn drop_node(self) {
        // SAFETY: The pointer originated from a Box and is therefore valid. The caller must ensure
        // that the value isn't referenced again after dropping.
        unsafe { drop(Box::from_non_null(self.0)) }
    }

    pub const fn as_non_null(self) -> NonNull<Node<T>> {
        self.0
    }

    pub const fn as_ptr(self) -> *mut Node<T> {
        self.0.as_ptr()
    }
}

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodePtr<T> {}

impl<T> PartialEq for NodePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
