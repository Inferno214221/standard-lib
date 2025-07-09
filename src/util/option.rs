pub(crate) trait OptionExtension<T> {
    fn unreachable(self) -> T;
}
 
impl<T> OptionExtension<T> for Option<T> {
    /// Acts similarly to [`Option::unwrap`] but with [`unreachable!`] in the none branch. This may
    /// be changed to [`unreachable_unchecked`](std::hint::unreachable_unchecked) in the future,
    /// possibly for only release builds.
    /// 
    /// This function does panic if used incorrectly, but no panics annotaions are used to allow it
    /// to pass the clippy lint. The whole semantics are that if used, the function indicates that
    /// None is impossible.
    fn unreachable(self) -> T {
        match self {
            Some(val) => val,
            None => unreachable!(),
        }
    }
}
