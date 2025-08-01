use std::io::{self, RawOsError};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use libc::{c_char, c_int, stat};

use crate::collections::contiguous::Vector;

pub struct File {
    fd: FileDescriptor,
    meta: FileMetadata,
}

pub(crate) struct FileDescriptor(pub c_int);

impl Deref for FileDescriptor {
    type Target = c_int;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO pub struct OpenOptions {}

pub struct FileMetadata {
    size: u64,
}

impl File {
    fn get_metadata(fd: i32) -> FileMetadata {
        let mut raw: MaybeUninit<stat> = MaybeUninit::uninit();
        if unsafe { libc::fstat(fd, raw.as_mut_ptr()) } == -1 {
            // TODO: Error handling
        }
        let raw = unsafe { raw.assume_init() };

        FileMetadata {
            size: raw.st_size as u64,
        }
    }

    pub fn open(file_path: &Path) -> Result<File, RawOsError> {
        let pathname: *const c_char = file_path.as_os_str().as_bytes().as_ptr().cast();
        let flags = libc::O_RDWR;

        match unsafe { libc::open(pathname, flags) } {
            -1 => Err(errno()),
            fd => Ok(File {
                fd: FileDescriptor(fd),
                meta: File::get_metadata(fd),
            }),
        }
    }

    pub fn create(file_path: &Path, mode: c_int) -> Result<File, RawOsError> {
        let pathname: *const c_char = file_path.as_os_str().as_bytes().as_ptr().cast();
        let flags: c_int = libc::O_RDWR | libc::O_CREAT | libc::O_EXCL;

        match unsafe { libc::open(pathname, flags, mode) } {
            -1 => Err(errno()),
            fd => Ok(File {
                fd: FileDescriptor(fd),
                meta: File::get_metadata(fd),
            }),
        }
    }

    pub fn open_or_create(file_path: &Path, mode: c_int) -> Result<File, RawOsError> {
        let pathname: *const c_char = file_path.as_os_str().as_bytes().as_ptr().cast();
        let flags: c_int = libc::O_RDWR | libc::O_CREAT;

        match unsafe { libc::open(pathname, flags, mode) } {
            -1 => Err(errno()),
            fd => Ok(File {
                fd: FileDescriptor(fd),
                meta: File::get_metadata(fd),
            }),
        }
    }

    pub fn read_into_buffer(&self, buf: &mut [u8]) -> Result<usize, RawOsError> {
        match unsafe { libc::read(*self.fd, buf.as_mut_ptr().cast(), buf.len()) } {
            -1 => Err(errno()),
            count => Ok(count as usize),
        }
    }

    pub fn read_all(&self) -> Result<Vector<u8>, RawOsError> {
        // This doesn't read the terminating byte at the moment.
        let size = self.meta.size as usize;
        let buf: Vector<u8> = Vector::with_cap(size);
        let (ptr, len, cap) = buf.into_parts();

        match unsafe { libc::read(*self.fd, ptr.as_ptr().add(len).cast(), size) } {
            -1 => {
                unsafe { drop(Vector::from_parts(ptr, len, cap)); }
                Err(errno())
            },
            count if size > count as usize => todo!("Repeat until all bytes are read!"),
            count => unsafe { Ok(Vector::from_parts(ptr, len + count as usize, cap)) },
        }
    }

    pub fn read_all_string(&self) -> Result<String, RawOsError> {
        Ok(String::from_utf8(self.read_all()?.into()).unwrap())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if unsafe { libc::close(*self.fd) } == -1 {
            match errno() {
                libc::EBADF => todo!(),
                libc::EINTR => todo!(),
                libc::EIO => todo!(),
                libc::ENOSPC => todo!(),
                libc::EDQUOT => todo!(),
                _ => (),
            }
        }
    }
}

fn errno() -> RawOsError {
    unsafe { io::Error::last_os_error().raw_os_error().unwrap_unchecked() }
}