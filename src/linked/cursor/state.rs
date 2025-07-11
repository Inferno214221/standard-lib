/// An enum to represent the state of a [`Cursor`](super::Cursor).
#[derive(Debug, PartialEq, Eq)]
pub enum State<'a, T> {
    /// The cursor holds an empty list and therefore doesn't point anywhere.
    Empty,
    /// The cursor is pointing to the 'ghost' element before the start of a list.
    Head,
    /// The cursor is pointing to the 'ghost' element after the end of a list.
    Tail,
    /// The cursor is pointing to a Node within the list, containing the borrowed value.
    Node(&'a T),
}

/// An enum to represent the state of a [`Cursor`](super::Cursor) while allowing for mutation.
#[derive(Debug, PartialEq, Eq)]
pub enum StateMut<'a, T> {
    /// The cursor holds an empty list and therefore doesn't point anywhere.
    Empty,
    /// The cursor is pointing to the 'ghost' element before the start of a list.
    Head,
    /// The cursor is pointing to the 'ghost' element after the end of a list.
    Tail,
    /// The cursor is pointing to a Node within the list, containing the mutably borrowed value.
    Node(&'a mut T),
}
