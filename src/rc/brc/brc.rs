use std::{borrow::Borrow, cmp::Ordering, fmt::{self, Debug, Display, Formatter, Pointer}, hash::{Hash, Hasher}, ops::Deref, rc::Rc, sync::Arc};


// Trait bounds are for the ZST backend itself so that it doesn't interfere with derive bounds.
pub trait Backend: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Default {
    type Inner<T: ?Sized>;

    fn new<T>(value: T) -> Self::Inner<T>;

    fn deref<T: ?Sized>(this: &Self::Inner<T>) -> &T;
    fn try_unwrap<T>(this: Self::Inner<T>) -> Result<T, Self::Inner<T>>;
    fn into_inner<T>(this: Self::Inner<T>) -> Option<T>;
    fn get_mut<T: ?Sized>(this: &mut Self::Inner<T>) -> Option<&mut T>;
    fn make_mut<T: Clone>(this: &mut Self::Inner<T>) -> &mut T;
    fn strong_count<T: ?Sized>(this: &Self::Inner<T>) -> usize;
    fn unwrap_or_clone<T: Clone>(this: Self::Inner<T>) -> T;
    fn clone<T: ?Sized>(this: &Self::Inner<T>) -> Self::Inner<T>;

    fn ptr_eq<T: ?Sized>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool;
    fn eq<T: ?Sized + PartialEq>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> bool;
    fn partial_cmp<T: ?Sized + PartialOrd>(
        this: &Self::Inner<T>,
        other: &Self::Inner<T>
    ) -> Option<Ordering>;
    fn cmp<T: ?Sized + Ord>(this: &Self::Inner<T>, other: &Self::Inner<T>) -> Ordering;

    fn from_iter<T, I: IntoIterator<Item = T>>(iter: I) -> Self::Inner<[T]>;

    fn from<T>(this: T) -> Self::Inner<T>;
    fn from_box<T>(this: Box<T>) -> Self::Inner<T>;

    fn from_slice<T: Copy>(this: &[T]) -> Self::Inner<[T]>;
    fn from_str(this: &str) -> Self::Inner<str>;

    fn from_vec<T>(this: Vec<T>) -> Self::Inner<[T]>;
    fn from_string(this: String) -> Self::Inner<str>;

    // Triomphe's Arc's repr(C) is not part of the public ABI, so we can't just copy the From<&str>
    // implementation for other unsized types.
}

/// A generic reference counted allocation, backed by the [`Backend`], `B`.
pub struct Brc<T: ?Sized, B: Backend> {
    pub(crate) inner: B::Inner<T>,
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

    pub fn is_unique(this: &Self) -> bool {
        Self::strong_count(this) == 0
    }

    pub fn ptr_eq(this: &Self, other: Self) -> bool {
        B::ptr_eq(&this.inner, &other.inner)
    }
}

impl<T: Clone, B: Backend> Brc<T, B> {
    pub fn unwrap_or_clone(this: Self) -> T {
        B::unwrap_or_clone(this.inner)
    }

    pub fn make_mut(this: &mut Self) -> &mut T {
        B::make_mut(&mut this.inner)
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
            inner: <B as Backend>::clone(&self.inner),
        }
    }
}

impl<T: ?Sized + PartialEq, B: Backend> PartialEq for Brc<T, B> {
    fn eq(&self, other: &Self) -> bool {
        <B as Backend>::eq(&self.inner, &other.inner)
    }
}

impl<T: ?Sized + Eq, B: Backend> Eq for Brc<T, B> {}

impl<T: ?Sized + PartialOrd, B: Backend> PartialOrd for Brc<T, B> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <B as Backend>::partial_cmp(&self.inner, &other.inner)
    }
}

impl<T: ?Sized + Ord, B: Backend> Ord for Brc<T, B> {
    fn cmp(&self, other: &Self) -> Ordering {
        <B as Backend>::cmp(&self.inner, &other.inner)
    }
}

impl<T: ?Sized + Hash, B: Backend> Hash for Brc<T, B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + Debug, B: Backend> Debug for Brc<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Brc")
            .field("inner", &self.deref())
            .finish()
    }
}

impl<T: ?Sized + Display, B: Backend> Display for Brc<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&&**self, f)
    }
}

impl<T: ?Sized, B: Backend> Pointer for Brc<T, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&&**self, f)
    }
}

impl<T, B: Backend> FromIterator<T> for Brc<[T], B> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Brc { inner: <B as Backend>::from_iter(iter) }
    }
}

impl<T, B: Backend> From<T> for Brc<T, B> {
    fn from(value: T) -> Self {
        Brc { inner: <B as Backend>::from(value) }
    }
}

impl<T, B: Backend> From<Box<T>> for Brc<T, B> {
    fn from(value: Box<T>) -> Self {
        Brc { inner: <B as Backend>::from_box(value) }
    }
}

impl<T: Copy, B: Backend> From<&[T]> for Brc<[T], B> {
    fn from(value: &[T]) -> Self {
        Brc { inner: <B as Backend>::from_slice(value) }
    }
}

impl<B: Backend> From<&str> for Brc<str, B> {
    fn from(value: &str) -> Self {
        Brc { inner: <B as Backend>::from_str(value) }
    }
}

impl<T, B: Backend> From<Vec<T>> for Brc<[T], B> {
    fn from(value: Vec<T>) -> Self {
        Brc { inner: <B as Backend>::from_vec(value) }
    }
}

impl<B: Backend> From<String> for Brc<str, B> {
    fn from(value: String) -> Self {
        Brc { inner: <B as Backend>::from_string(value) }
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

            fn unwrap_or_clone<T: Clone>(this: Self::Inner<T>) -> T {
                $rc_type::unwrap_or_clone(this)
            }

            fn get_mut<T: ?Sized>(this: &mut Self::Inner<T>) -> Option<&mut T> {
                $rc_type::get_mut(this)
            }

            fn make_mut<T: ?Sized + Clone>(this: &mut Self::Inner<T>) -> &mut T {
                $rc_type::make_mut(this)
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

            fn partial_cmp<T: ?Sized + PartialOrd>(
                this: &Self::Inner<T>,
                other: &Self::Inner<T>
            ) -> Option<Ordering> {
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

            fn from_iter<T, I: IntoIterator<Item = T>>(iter: I) -> Self::Inner<[T]> {
                $rc_type::<[T]>::from_iter(iter)
            }

            fn from<T>(this: T) -> Self::Inner<T> {
                $rc_type::<T>::from(this)
            }

            fn from_box<T>(this: Box<T>) -> Self::Inner<T> {
                $rc_type::<T>::from(this)
            }
            fn from_slice<T: Copy>(this: &[T]) -> Self::Inner<[T]> {
                $rc_type::<[T]>::from(this)
            }

            fn from_str(this: &str) -> Self::Inner<str> {
                $rc_type::<str>::from(this)
            }

            fn from_vec<T>(this: Vec<T>) -> Self::Inner<[T]> {
                $rc_type::<[T]>::from(this)
            }

            fn from_string(this: String) -> Self::Inner<str> {
                $rc_type::<str>::from(this)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UseRc;
impl_backend!(Rc, UseRc);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UseArc;
impl_backend!(Arc, UseArc);