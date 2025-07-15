use std::num::NonZero;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct Length(pub NonZero<usize>);

impl Length {
    pub const fn checked_add(self, other: usize) -> Option<Length> {
        Length::wrap_non_zero(self.0.checked_add(other))
    }

    pub const fn checked_sub(self, other: usize) -> Option<Length> {
        Length::wrap_non_zero(match self.0.get().checked_sub(other) {
            Some(res) => NonZero::new(res),
            None => None,
        })
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }

    pub const fn wrap_non_zero(value: Option<NonZero<usize>>) -> Option<Length> {
        match value {
            Some(res) => Some(Length(res)),
            None => None,
        }
    }

    pub const fn new(value: usize) -> Option<Length> {
        Length::wrap_non_zero(NonZero::new(value))
    }

    pub const unsafe fn new_unchecked(value: usize) -> Length {
        Length(unsafe { NonZero::new_unchecked(value) })
    }
}

pub(crate) const ONE: Length = Length(NonZero::<usize>::MIN);