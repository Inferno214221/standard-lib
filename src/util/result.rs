use std::error::Error;

pub(crate) trait ResultExtension<T, E: Error> {
    /// A method similar to [`Result::unwrap`], except that it applies only to types which implement
    /// [`Error`] and panics with the message of the error itself.
    ///
    /// # Panics
    /// Panics if the [`Result`] is an [`Err`].
    fn throw(self) -> T;
}

impl<T, E: Error> ResultExtension<T, E> for Result<T, E> {
    fn throw(self) -> T {
        match self {
            Ok(val) => val,
            Err(error) => panic!("{}", error),
        }
    }
}
