use std::ptr::NonNull;

pub(crate) type Link<T> = Option<NodeRef<T>>;

// NOTE: This implementation uses Box<T> rather than alloc to allocate space on the heap, because
// Box<T> has the special property that dereferencing it allows a value to be moved out of the heap.

#[derive(Debug)]
pub(crate) struct NodeRef<T>(NonNull<Node<T>>);

impl<T> NodeRef<T> {
    pub fn value<'a>(&self) -> &'a T {
        unsafe { &(self.0.as_ref()).value }
        // unsafe { &(*self.0.as_ptr()).value }
    }

    pub fn value_mut<'a>(&mut self) -> &'a mut T {
        unsafe { &mut (self.0.as_mut()).value }
        // unsafe { &mut (*self.0.as_ptr()).value }
    }

    pub fn prev(&self) -> &Link<T> {
        unsafe { &(*self.0.as_ptr()).prev }
    }

    pub fn prev_mut(&self) -> &mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).prev }
    }

    pub fn next(&self) -> &Link<T> {
        unsafe { &(*self.0.as_ptr()).next }
    }

    pub fn next_mut(&self) -> &mut Link<T> {
        unsafe { &mut (*self.0.as_ptr()).next }
    }

    pub fn from_node(node: Node<T>) -> NodeRef<T> {
        NodeRef(Box::into_non_null(Box::new(node)))
    }

    pub fn take_node(self) -> Node<T> {
        unsafe { *Box::from_non_null(self.0) }
    }

    pub fn as_non_null(self) -> NonNull<Node<T>> {
        self.0
    }
}

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
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
    pub next: Link<T>
}