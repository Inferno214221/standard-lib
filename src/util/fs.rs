#![cfg(target_os = "linux")]

use std::io::{self, RawOsError};

use libc::{c_int, c_void};

pub fn err_no() -> RawOsError {
    // SAFETY: raw_os_error guarantees Some if constructed from last_os_error.
    unsafe { io::Error::last_os_error().raw_os_error().unwrap_unchecked() }
}

/// # Safety
/// - `fd` must be a valid, open directory file descriptor.
/// - `dirp` must point to a valid, writable buffer of at least `bytes` length.
/// - The buffer must be properly aligned for the `linux_dirent64` structure.
/// - The caller must ensure the buffer remains valid for the duration of the syscall.
pub unsafe fn getdents(fd: c_int, dirp: *mut c_void, bytes: usize) -> isize {
    // SAFETY: Caller guarantees fd is valid and dirp points to a valid buffer of `bytes` length.
    // SYS_getdents64 will fill the buffer with linux_dirent64 structures.
    unsafe { libc::syscall(libc::SYS_getdents64, fd, dirp, bytes) as isize }
}
