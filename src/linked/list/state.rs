// TODO: Move to linked::cursor

#[derive(Debug, PartialEq, Eq)]
pub enum State<'a, T> {
    Empty,
    Head,
    Tail,
    Node(&'a T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum StateMut<'a, T> {
    Empty,
    Head,
    Tail,
    Node(&'a mut T),
}
