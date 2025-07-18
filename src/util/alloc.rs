use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ZeroSizedType;

#[derive(Debug, Clone)]
pub struct CountedDrop(pub Rc<RefCell<usize>>);

impl CountedDrop {
    pub fn new(value: usize) -> CountedDrop {
        CountedDrop(Rc::new(RefCell::new(value)))
    }
}

impl Deref for CountedDrop {
    type Target = Rc<RefCell<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CountedDrop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for CountedDrop {
    fn drop(&mut self) {
        self.0.replace_with(|v| *v + 1);
    }
}
