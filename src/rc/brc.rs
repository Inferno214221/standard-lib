use std::{borrow::Borrow, cmp::Ordering, ops::Deref, rc::Rc, sync::Arc};

use derive_more::Display;

pub trait Backend {
    type Inner<T: ?Sized>;

    fn new<T>(value: T) -> Self::Inner<T>;

    fn try_unwrap<T>(this: Self::Inner<T>) -> Result<T, Self::Inner<T>>;

    fn into_inner<T>(this: Self::Inner<T>) -> Option<T>;

    fn get_mut<T: ?Sized>(this: &mut Self::Inner<T>) -> Option<&mut T>;

    fn strong_count<T: ?Sized>(this: &Self::Inner<T>) -> usize;

    fn ptr_eq<T: ?Sized>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool;

    fn eq<T: ?Sized + PartialEq>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool;

    fn partial_cmp<T: ?Sized + PartialOrd>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Option<Ordering>;

    fn cmp<T: ?Sized + Ord>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Ordering;

    fn deref<T: ?Sized>(this: &Self::Inner<T>) -> &T;

    fn clone<T: ?Sized>(this: &Self::Inner<T>) -> Self::Inner<T>;
}

/// A generic reference counted allocation, backed by the [`Backend`], `B`.
#[derive(Debug, Display)]
pub struct Brc<T: ?Sized, B: Backend> {
    pub inner: B::Inner<T>,
}

impl<T, B: Backend> Brc<T, B> {
    pub fn new(value: T) -> Brc<T, B> {
        Brc {
            inner: B::new(value),
        }
    }

    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        match B::try_unwrap(this.inner) {
            Ok(inner) => Ok(inner),
            Err(value) => Err(Brc {
                inner: value,
            }),
        }
    }

    pub fn into_inner(this: Self) -> Option<T> {
        B::into_inner(this.inner)
    }
}

impl<T: ?Sized, B: Backend> Brc<T, B> {
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        B::get_mut(&mut this.inner)
    }

    pub fn strong_count(this: &Self) -> usize {
        B::strong_count(&this.inner)
    }

    pub fn ptr_eq(this: &Self, other: Self) -> bool {
        B::ptr_eq(&this.inner, &other.inner)
    }
}

impl<T: ?Sized, B: Backend> Deref for Brc<T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        B::deref(&self.inner)
    }
}

impl<T: ?Sized, B: Backend> AsRef<T> for Brc<T, B> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized, B: Backend> Borrow<T> for Brc<T, B> {
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized, B: Backend> Clone for Brc<T, B> {
    fn clone(&self) -> Self {
        Brc {
            inner: B::clone(&self.inner),
        }
    }
}

impl<T: ?Sized + PartialEq, B: Backend> PartialEq for Brc<T, B> {
    fn eq(&self, other: &Self) -> bool {
        B::eq(&self.inner, &other.inner)
    }
}

impl<T: ?Sized + Eq, B: Backend> Eq for Brc<T, B> {}

impl<T: ?Sized + PartialOrd, B: Backend> PartialOrd for Brc<T, B> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        B::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: ?Sized + Ord, B: Backend> Ord for Brc<T, B> {
    fn cmp(&self, other: &Self) -> Ordering {
        B::cmp(&self.inner, &other.inner)
    }
}

macro_rules! impl_backend {
    ($rc_type:tt, $new_type:tt) => {
        impl Backend for $new_type {
            type Inner<T: ?Sized> = $rc_type<T>;

            fn new<T>(value: T) -> Self::Inner<T> {
                $rc_type::new(value)
            }

            fn try_unwrap<T>(this: Self::Inner<T>) -> Result<T, Self::Inner<T>> {
                $rc_type::try_unwrap(this)
            }

            fn into_inner<T>(this: Self::Inner<T>) -> Option<T> {
                $rc_type::into_inner(this)
            }

            fn get_mut<T: ?Sized>(this: &mut Self::Inner<T>) -> Option<&mut T> {
                $rc_type::get_mut(this)
            }

            fn strong_count<T: ?Sized>(this: &Self::Inner<T>) -> usize {
                $rc_type::strong_count(this)
            }

            fn ptr_eq<T: ?Sized>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool {
                $rc_type::ptr_eq(this, other)
            }

            fn eq<T: ?Sized + PartialEq>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool {
                $rc_type::eq(this, other)
            }

            fn partial_cmp<T: ?Sized + PartialOrd>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Option<Ordering> {
                $rc_type::partial_cmp(this, other)
            }

            fn cmp<T: ?Sized + Ord>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Ordering {
                $rc_type::cmp(this, other)
            }

            fn deref<T: ?Sized>(this: &Self::Inner<T>) -> &T {
                $rc_type::deref(this)
            }

            fn clone<T: ?Sized>(this: &Self::Inner<T>) -> Self::Inner<T> {
                $rc_type::clone(this)
            }
        }
    };
}

pub struct UseRc;
impl_backend!(Rc, UseRc);

pub struct UseArc;
impl_backend!(Arc, UseArc);