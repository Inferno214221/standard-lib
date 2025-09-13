use std::{any, fmt::{self, Debug, Formatter}};

pub struct DebugRaw(pub String);

impl Debug for DebugRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn raw_type_name<T>() -> DebugRaw {
    DebugRaw(
        any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("")
            .to_string()
    )
}