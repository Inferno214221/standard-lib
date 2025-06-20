use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

// NOTE: This implementation uses Box<T> rather than alloc to allocate space on the heap, because
// Box<T> has the special property that dereferencing it allows a value to be moved out of the heap.

pub struct DLinkedList<T> {
    pub(crate) head: Link<T>,
    pub(crate) tail: Link<T>,
    pub(crate) len: usize,
    _phantom: PhantomData<T>
}

type Link<T> = Option<NonNull<Node<T>>>;

pub(crate) struct Node<T> {
    pub(crate) value: T,
    pub(crate) prev: Link<T>,
    pub(crate) next: Link<T>
}

impl<T> DLinkedList<T> {
    pub fn new() -> DLinkedList<T> {
        DLinkedList {
            head: None,
            tail: None,
            len: 0,
            _phantom: PhantomData
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn check_index(&self, index: usize) {
        assert!(
            index <= self.len,
            "Index {} out of bounds for DLinkedList with len {}", index, self.len
        );
    }

    pub fn push_back(&mut self, value: T) {
        let ptr = Box::into_non_null(Box::new(
            Node {
                value,
                prev: None,
                next: None
            }
        ));

        if let Some(old_tail) = self.tail {
            unsafe {
                (*old_tail.as_ptr()).next = Some(ptr);
                (*ptr.as_ptr()).prev = Some(old_tail);
            };
            self.tail = Some(ptr);
        } else {
            self.head = Some(ptr);
            self.tail = Some(ptr);
        }
        self.len += 1;
    }

    pub fn push_front(&mut self, value: T) {
        let ptr = Box::into_non_null(Box::new(
            Node {
                value,
                prev: None,
                next: None
            }
        ));

        if let Some(old_head) = self.head {
            unsafe {
                (*old_head.as_ptr()).prev = Some(ptr);
                (*ptr.as_ptr()).next = Some(old_head);
            };
            self.head = Some(ptr);
        } else {
            self.head = Some(ptr);
            self.tail = Some(ptr);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        match self.tail {
            Some(old_tail) if self.len == 1 => {
                debug_assert!(self.head == self.tail);

                let node = unsafe { *Box::from_non_null(old_tail) };

                self.head = None;
                self.tail = None;
                self.len = 0;

                Some(node.value)
            },
            Some(old_tail) => {
                debug_assert!(self.head != self.tail);
                debug_assert!(self.len > 1);

                let node = unsafe { *Box::from_non_null(old_tail) };

                let new_tail = node.prev.unwrap();
                self.tail = Some(new_tail);
                unsafe { (*new_tail.as_ptr()).next = None };
                self.len -= 1;

                Some(node.value)
            },
            _ => {
                debug_assert!(self.head == None);
                debug_assert!(self.len == 0);

                None
            },
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.head {
            Some(old_head) if self.len == 1 => {
                debug_assert!(self.head == self.tail);

                let node = unsafe { *Box::from_non_null(old_head) };

                self.head = None;
                self.tail = None;
                self.len = 0;

                Some(node.value)
            },
            Some(old_head) => {
                debug_assert!(self.head != self.tail);
                debug_assert!(self.len > 1);

                let node = unsafe { *Box::from_non_null(old_head) };

                let new_head = node.prev.unwrap();
                self.head = Some(new_head);
                unsafe { (*new_head.as_ptr()).prev = None };
                self.len -= 1;

                Some(node.value)
            },
            _ => {
                debug_assert!(self.head == None);
                debug_assert!(self.len == 0);

                None
            },
        }
    }

    pub(crate) fn seek(&self, index: usize) -> &Node<T> {
        if index < self.len / 2 {
            self.seek_fwd(index)
        } else {
            self.seek_bwd(index)
        }
    }

    pub(crate) fn seek_mut(&mut self, index: usize) -> &mut Node<T> {
        if index < self.len / 2 {
            self.seek_fwd_mut(index)
        } else {
            self.seek_bwd_mut(index)
        }
    }

    pub(crate) fn seek_fwd(&self, index: usize) -> &Node<T> {
        // TODO: can't call unwrap without handling the possibility of len = 0.
        let mut curr = self.head.unwrap();
        for _ in 0..index {
            curr = unsafe { curr.as_ref().next.unwrap() };
        }
        unsafe { curr.as_ref() }
    }

    pub(crate) fn seek_bwd(&self, index: usize) -> &Node<T> {
        let mut curr = self.tail.unwrap();
        for _ in 1..(self.len - index) {
            curr = unsafe { curr.as_ref().prev.unwrap() };
        }
        unsafe { curr.as_ref() }
    }

    pub(crate) fn seek_fwd_mut(&mut self, index: usize) -> &mut Node<T> {
        let mut curr = self.head.unwrap();
        for _ in 0..index {
            curr = unsafe { curr.as_ref().next.unwrap() };
        }
        unsafe { curr.as_mut() }
    }

    pub(crate) fn seek_bwd_mut(&mut self, index: usize) -> &mut Node<T> {
        let mut curr = self.tail.unwrap();
        for _ in 1..(self.len - index) {
            curr = unsafe { curr.as_ref().prev.unwrap() };
        }
        unsafe { curr.as_mut() }
    }

    pub fn get(&self, index: usize) -> &T {
        self.check_index(index);

        &self.seek(index).value
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.check_index(index);

        &mut self.seek_mut(index).value
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }
}

impl<T> Default for DLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for DLinkedList<T> {
    fn drop(&mut self) {
        self.head.map(|head| {
            let mut curr = unsafe { *Box::from_non_null(head) };
            loop {
                drop(curr.value);
                match curr.next {
                    Some(next) => {
                        curr = unsafe { *Box::from_non_null(next) }
                    },
                    None => break,
                }
            }
        });
    }
}

// impl Extend for DLinkedList

impl<T: Debug> Debug for DLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DLinkedList")
            .field("len", &self.len)
            .field(
                "elements",
                &(self.iter().map(
                    |v| format!("({v:?}) -> ")
                ).collect::<String>() + "End")
            )
            .finish()
    }
}

pub struct IntoIter<T>(Link<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|ptr| {
            // Use a box to move the value and clean up.
            let node = unsafe { *Box::from_non_null(ptr) };
            self.0 = node.next;
            node.value
        })
    }
}

impl<T> IntoIterator for DLinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.head)
    }
}

pub struct Iter<'a, T>(&'a Link<T>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|ptr| {
            let node = unsafe { ptr.as_ref() };
            self.0 = &node.next;
            &node.value
        })
    }
}

impl<'a, T> IntoIterator for &'a DLinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(&self.head)
    }
}

pub struct IterMut<'a, T>(&'a Link<T>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|mut ptr| {
            let node = unsafe { ptr.as_mut() };
            self.0 = &mut node.next;
            &mut node.value
        })
    }
}

impl<'a, T> IntoIterator for &'a mut DLinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut(&mut self.head)
    }
}