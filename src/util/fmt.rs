use std::fmt::{self, Debug, Formatter};

pub struct DebugRaw(pub String);

impl Debug for DebugRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}