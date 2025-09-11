use std::ffi::CString;
use std::io::RawOsError;
use std::marker::PhantomData;

use libc::{O_APPEND, O_CREAT, O_EXCL, O_NOATIME, O_NOFOLLOW, O_SYNC, O_TRUNC, c_int};

use super::{File, AccessMode};
use crate::fs::path::{Abs, Path};
use crate::fs::{Fd, util};

#[derive(Debug, Clone)]
pub struct OpenOptions<Access: AccessMode> {
    // TODO: Should I make this pub again so that it can be constructed manually? Maybe just add a
    // new method?
    pub(crate) _access: PhantomData<fn() -> Access>,
    pub create: Option<Create>,
    pub mode: Option<u16>,
    pub append: Option<bool>,
    pub force_sync: Option<bool>,
    pub update_access_time: Option<bool>,
    pub follow_links: Option<bool>,
    pub extra_flags: Option<i32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Create {
    No,
    #[default]
    IfAbsent,
    OrClear,
    Require,
}

impl<A: AccessMode> OpenOptions<A> {
    pub(crate) fn flags(&self) -> c_int {
        let mut flags = A::FLAGS;
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
        flags | self.extra_flags.unwrap_or_default()
    }

    pub fn new() -> OpenOptions<A> {
        OpenOptions::<A>::default()
    }

    pub fn open<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<File<A>, RawOsError> {
        let pathname = CString::from(file_path.as_ref().to_owned());
        // TODO: Permission builder of some type?
        let mode = self.mode.unwrap_or(0o644) as c_int;

        match unsafe { libc::open(pathname.as_ptr().cast(), self.flags(), mode) } {
            -1 => Err(util::err_no()),
            fd => Ok(File::<A> {
                _access: PhantomData,
                fd: Fd(fd),
            }),
        }
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

    pub const fn extra_flags(&mut self, value: i32) -> &mut Self {
        self.extra_flags = Some(value);
        self
    }
}

// The Default derive macro doesn't like my spooky zero-variant enums.
impl<A: AccessMode> Default for OpenOptions<A> {
    fn default() -> Self {
        Self {
            _access: Default::default(),
            create: Default::default(),
            mode: Default::default(),
            append: Default::default(),
            force_sync: Default::default(),
            update_access_time: Default::default(),
            follow_links: Default::default(),
            extra_flags: Default::default()
        }
    }
}
