use super::Vector;
use crate::collections::contiguous::Array;
#[doc(inline)]
pub use crate::collections::contiguous::array::IntoIter;

impl<T> IntoIterator for Vector<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Array::from(self).into_iter()
    }
}
