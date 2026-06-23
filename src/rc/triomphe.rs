use std::ops::Deref;

use triomphe::{Arc, UniqueArc};

use crate::rc::brc::Backend;

pub struct UseTArc;

impl Backend for UseTArc {
    type Inner<T: ?Sized> = Arc<T>;

    fn new<T>(value: T) -> Self::Inner<T> {
        Arc::new(value)
    }

    fn try_unwrap<T>(this: Self::Inner<T>) -> Result<T, Self::Inner<T>> {
        Arc::try_unwrap(this)
    }

    fn into_inner<T>(this: Self::Inner<T>) -> Option<T> {
        Some(UniqueArc::into_inner(Arc::into_unique(this)?))
    }

    fn get_mut<T: ?Sized>(this: &mut Self::Inner<T>) -> Option<&mut T> {
        Arc::get_mut(this)
    }

    fn strong_count<T: ?Sized>(this: &Self::Inner<T>) -> usize {
        Arc::strong_count(this)
    }

    fn ptr_eq<T: ?Sized>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool {
        Arc::ptr_eq(this, other)
    }

    fn eq<T: ?Sized + PartialEq>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool {
        Arc::eq(this, other)
    }

    fn partial_cmp<T: ?Sized + PartialOrd>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Option<std::cmp::Ordering> {
        Arc::partial_cmp(this, other)
    }

    fn cmp<T: ?Sized + Ord>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> std::cmp::Ordering {
        Arc::cmp(this, other)
    }

    fn deref<T: ?Sized>(this: &Self::Inner<T>) -> &T {
        Arc::deref(this)
    }

    fn clone<T: ?Sized>(this: &Self::Inner<T>) -> Self::Inner<T> {
        Arc::clone(this)
    }
}