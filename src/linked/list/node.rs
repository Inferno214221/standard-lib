use std::ptr::NonNull;

pub(crate) type Link<T> = Option<NodeRef<T>>;

// NOTE: This implementation uses Box<T> rather than alloc to allocate space on the heap, because
// Box<T> has the special property that dereferencing it allows a value to be moved out of the heap.

#[derive(Debug)]
pub(crate) struct NodeRef<T>(pub NonNull<Node<T>>);

impl<T> NodeRef<T> {
    pub const fn value<'a>(&self) -> &'a T {
        unsafe { &(self.0.as_ref()).value }
        // unsafe { &(*self.0.as_ptr()).value }
    }

    pub const fn value_mut<'a>(&mut self) -> &'a mut T {
        unsafe { &mut (self.0.as_mut()).value }
        // unsafe { &mut (*self.0.as_ptr()).value }
    }

    pub fn prev<'a>(&self) -> &'a Link<T> {
        unsafe { &(*self.0.as_ptr()).prev }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn prev_mut<'a>(&self) -> &'a mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).prev }
    }

    pub fn next<'a>(&self) -> &'a Link<T> {
        unsafe { &(*self.0.as_ptr()).next }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn next_mut<'a>(&self) -> &'a mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).next }
    }

    pub fn from_node(node: Node<T>) -> NodeRef<T> {
        NodeRef(Box::into_non_null(Box::new(node)))
    }

    pub fn take_node(self) -> Node<T> {
        unsafe { *Box::from_non_null(self.0) }
    }

    pub const fn as_non_null(self) -> NonNull<Node<T>> {
        self.0
    }

    pub const fn as_ptr(self) -> *mut Node<T> {
        self.0.as_ptr()
    }
}

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeRef<T> {}

impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub(crate) struct Node<T> {
    pub value: T,
    pub prev: Link<T>,
    pub next: Link<T>,
}
