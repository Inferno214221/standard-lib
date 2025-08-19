use std::ffi::CStr;

use crate::{collections::contiguous::Vector, fs::{file::FileType, syscall, Fd}};

#[repr(C)]
struct DirEntrySized {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u8,
    pub d_type: u8,
}

#[derive(Debug)]
pub struct DirEntry {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u8,
    pub file_type: FileType,
    pub name: String,
}

const BUFFER_SIZE: usize = 1024;

pub struct DirEntries {
    pub(crate) fd: Fd,
    pub(crate) buf: Option<[i8; BUFFER_SIZE]>,
    pub(crate) index: usize,
}

// TODO: need to ensure that undersized buffer is handled.

impl Iterator for DirEntries {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.buf {
            Some(buf) => {
                // TODO: If count is exceeded, revert to None. If getdents returns 0 without failing, stop.
                // if unsafe { buf.as_ptr().cast::<DirEntrySized>().as_ref_unchecked().d_reclen == 0 } {
                //     let ptr = unsafe { buf.as_mut_ptr().cast() };
                //     let count = match unsafe { syscall::getdents(self.fd, ptr, BUFFER_SIZE) } {
                //         -1 => todo!("{}", syscall::err_no()),
                //         count => count as usize,
                //     };
                // }
                let head = unsafe { buf.as_ptr().add(self.index) };
                let sized = unsafe { head.cast::<DirEntrySized>().as_ref_unchecked() };
                Some(DirEntry {
                    d_ino: sized.d_ino,
                    d_off: sized.d_off,
                    d_reclen: sized.d_reclen as u8, // TODO: Why isn't this correct everywhere?
                    file_type: FileType::Regular, // todo!("{}", sized.d_type),
                    name: unsafe { CStr::from_ptr(head.add(18)).to_str().unwrap().to_owned() },
                })
            },
            None => {
                let mut buf = [0; 1024];
                let ptr = unsafe { buf.as_mut_ptr().cast() };
                let count = match unsafe { syscall::getdents(self.fd, ptr, BUFFER_SIZE) } {
                    -1 => todo!("{}", syscall::err_no()),
                    count => count as usize,
                };
                let head = unsafe { buf.as_ptr().add(self.index) };
                let sized = unsafe { head.cast::<DirEntrySized>().as_ref_unchecked() };

                self.buf = Some(buf);
                Some(DirEntry {
                    d_ino: sized.d_ino,
                    d_off: sized.d_off,
                    d_reclen: sized.d_reclen as u8, // TODO: Why isn't this correct everywhere?
                    file_type: FileType::Regular, // todo!("{}", sized.d_type),
                    name: unsafe { CStr::from_ptr(head.add(18)).to_str().unwrap().to_owned() },
                })
            },
        }
    }
}
