use std::io::{self, RawOsError};

pub fn err_no() -> RawOsError {
    // SAFETY: raw_os_error guarantees Some if constructed from last_os_error.
    unsafe { io::Error::last_os_error().raw_os_error().unwrap_unchecked() }
}