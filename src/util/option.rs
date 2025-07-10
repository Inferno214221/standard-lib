use std::hint;

pub(crate) trait OptionExtension<T> {
    unsafe fn unreachable(self) -> T;
}
 
impl<T> OptionExtension<T> for Option<T> {
    /// Acts similarly to [`Option::unwrap`] but with [`unreachable!`] in the none branch for dev
    /// and [`unreachable_unchecked`](hint::unreachable_unchecked) for release builds.
    /// 
    /// This function does panic if used incorrectly, but no panics annotaions are used to allow it
    /// to pass the clippy lint. The whole semantics are that if used, the function indicates that
    /// None is impossible. The same goes with safety docs.
    unsafe fn unreachable(self) -> T {
        match self {
            Some(val) => val,
            None if cfg!(debug_assertions) => unreachable!(),
            // SAFETY: It is the responsibility of the caller to ensure that None is impossible when
            // invoking this method.
            None => unsafe { hint::unreachable_unchecked() },
        }
    }
}
