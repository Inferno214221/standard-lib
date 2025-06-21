use super::{DLinkedList, NodeRef, Node};

pub struct Cursor<T> {
    pub(crate) list: DLinkedList<T>,
    pub(crate) curr: NodeRef<T>
}

impl<T> Cursor<T> {
    pub fn as_list(self) -> DLinkedList<T> {
        self.list
    }

    pub fn current(&self) -> &T {
        self.curr.value()
    }

    pub fn current_mut(&mut self) -> &mut T {
        self.curr.value_mut()
    }

    pub fn peek_next(&self) -> Option<&T> {
        self.curr.next().as_ref().map(|next| next.value())
    }

    pub fn peek_next_mut(&mut self) -> Option<&mut T> {
        self.curr.next_mut().as_mut().map(|next| next.value_mut())
    }

    // TODO: should this return self??
    pub fn move_next(&mut self) -> Option<&mut Self> {
        match self.curr.next() {
            Some(next) => {
                self.curr = next.clone();
                Some(self)
            },
            None => None,
        }
    }

    pub fn peek_prev(&self) -> Option<&T> {
        self.curr.prev().as_ref().map(|prev| prev.value())
    }

    pub fn peek_prev_mut(&mut self) -> Option<&mut T> {
        self.curr.prev_mut().as_mut().map(|prev| prev.value_mut())
    }

    // TODO: should this return self??
    pub fn move_prev(&mut self) -> Option<&mut Self> {
        match self.curr.prev() {
            Some(prev) => {
                self.curr = prev.clone();
                Some(self)
            },
            None => None,
        }
    }

    pub fn insert_next(&mut self, value: T) {
        let node = NodeRef::from_node(Node {
            value,
            prev: Some(self.curr),
            next: self.curr.next().clone()
        });

        match self.curr.next() {
            Some(next) => *next.prev_mut() = Some(node),
            None => self.list.tail = Some(node),
        }

        *self.curr.next_mut() = Some(node);
    }

    pub fn insert_prev(&mut self, value: T) {
        let node = NodeRef::from_node(Node {
            value,
            prev: self.curr.prev().clone(),
            next: Some(self.curr)
        });

        match self.curr.prev() {
            Some(prev) => *prev.next_mut() = Some(node),
            None => self.list.head = Some(node),
        }

        *self.curr.prev_mut() = Some(node);
    }

    // TODO: redirect most to functions list with some extra handling.
}