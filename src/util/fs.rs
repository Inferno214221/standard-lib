#![cfg(target_os = "linux")]

use std::io::{self, RawOsError};

use libc::{c_int, c_void};

pub fn err_no() -> RawOsError {
    // SAFETY: raw_os_error guarantees Some if constructed from last_os_error.
    unsafe { io::Error::last_os_error().raw_os_error().unwrap_unchecked() }
}

pub unsafe fn getdents(fd: c_int, dirp: *mut c_void, bytes: usize) -> isize {
    unsafe { libc::syscall(libc::SYS_getdents64, fd, dirp, bytes) as isize }
}
