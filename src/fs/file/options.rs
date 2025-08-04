use std::{io::RawOsError, os::unix::ffi::OsStrExt, path::Path};

use libc::{c_char, c_int, O_APPEND, O_CREAT, O_EXCL, O_NOATIME, O_NOFOLLOW, O_RDONLY, O_RDWR, O_SYNC, O_TRUNC, O_WRONLY};

use crate::util::syscall;

use super::File;

#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    pub access: Option<AccessMode>,
    pub create: Option<Create>,
    pub mode: Option<u16>,
    pub append: Option<bool>,
    pub force_sync: Option<bool>,
    pub update_access_time: Option<bool>,
    pub follow_links: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AccessMode {
    Read,
    Write,
    #[default]
    ReadWrite,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Create {
    No,
    #[default]
    IfAbsent,
    OrClear,
    Require,
}

impl OpenOptions {
    pub(crate) fn flags(&self) -> c_int {
        let mut flags: c_int = match self.access.unwrap_or_default() {
            AccessMode::Read => O_RDONLY,
            AccessMode::Write => O_WRONLY,
            AccessMode::ReadWrite => O_RDWR,
        };
        match &self.create.unwrap_or_default() {
            Create::No => (),
            Create::IfAbsent => flags |= O_CREAT,
            Create::OrClear =>  flags |= O_CREAT | O_TRUNC,
            Create::Require =>  flags |= O_CREAT | O_EXCL,
        }
        if self.append.unwrap_or(false) {
            flags |= O_APPEND;
        }
        if self.force_sync.unwrap_or(false) {
            flags |= O_SYNC;
        }
        if !self.update_access_time.unwrap_or(true) {
            flags |= O_NOATIME;
        }
        if !self.follow_links.unwrap_or(true) {
            flags |= O_NOFOLLOW;
        }
        flags
    }

    pub fn new() -> OpenOptions {
        OpenOptions::default()
    }

    pub fn open(&self, file_path: &Path) -> Result<File, RawOsError> {
        let pathname: *const c_char = file_path.as_os_str().as_bytes().as_ptr().cast();

        match unsafe { libc::open(pathname, self.flags(), self.mode.unwrap_or(0o644) as c_int) } {
            -1 => Err(syscall::err_no()),
            fd => Ok(File { fd }),
        }
    }

    pub const fn readonly(&mut self) -> &mut Self {
        self.access = Some(AccessMode::Read);
        self
    }

    pub const fn writeonly(&mut self) -> &mut Self {
        self.access = Some(AccessMode::Write);
        self
    }

    pub const fn read_write(&mut self) -> &mut Self {
        self.access = Some(AccessMode::ReadWrite);
        self
    }

    pub const fn if_present(&mut self) -> &mut Self {
        self.create = Some(Create::No);
        self
    }

    pub const fn create_if_absent(&mut self) -> &mut Self {
        self.create = Some(Create::IfAbsent);
        self
    }

    pub const fn create_or_clear(&mut self) -> &mut Self {
        self.create = Some(Create::OrClear);
        self
    }

    pub const fn create_only(&mut self) -> &mut Self {
        self.create = Some(Create::Require);
        self
    }

    pub const fn mode(&mut self, value: u16) -> &mut Self {
        self.mode = Some(value);
        self
    }

    pub const fn append(&mut self, value: bool) -> &mut Self {
        self.append = Some(value);
        self
    }

    pub const fn force_sync(&mut self, value: bool) -> &mut Self {
        self.force_sync = Some(value);
        self
    }

    pub const fn update_access_time(&mut self, value: bool) -> &mut Self {
        self.update_access_time = Some(value);
        self
    }

    pub const fn follow_links(&mut self, value: bool) -> &mut Self {
        self.follow_links = Some(value);
        self
    }
}