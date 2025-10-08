use std::fmt;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::io::RawOsError;
use std::marker::PhantomData;

use libc::{O_APPEND, O_NOATIME, O_NOFOLLOW, O_SYNC, c_int};

use super::{File, AccessMode};
use crate::fs::dir::DirEntry;
use crate::fs::file::{Create, CreateIfMissing, CreateOrEmpty, NoCreate, OpenMode, ReadOnly, ReadWrite, WriteOnly};
use crate::fs::path::{Abs, Path};
use crate::fs::{Directory, Fd, Rel};
use crate::util;

/// A builder struct to help with opening files, using customizable options and logical defaults.
/// Available via [`File::options`] to avoid additional use statements.
// TODO: More docs here.
#[derive(Clone)]
pub struct OpenOptions<Access: AccessMode, Open: OpenMode> {
    pub(crate) _access: PhantomData<fn() -> Access>,
    pub(crate) _open: PhantomData<fn() -> Open>,
    pub mode: Option<u16>,
    pub append: Option<bool>,
    pub force_sync: Option<bool>,
    pub update_access_time: Option<bool>,
    pub follow_links: Option<bool>,
    pub extra_flags: Option<i32>,
}

impl<A: AccessMode, O: OpenMode> OpenOptions<A, O> {
    pub(crate) fn flags(&self) -> c_int {
        let mut flags = A::FLAGS | O::FLAGS;
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

    pub fn new() -> OpenOptions<A, O> {
        OpenOptions::<A, O>::default()
    }

    pub fn open<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<File<A>, RawOsError> {
        let pathname = CString::from(file_path.as_ref().to_owned());
        // TODO: Permission builder of some type?
        let mode = self.mode.unwrap_or(0o644) as c_int;

        match unsafe { libc::open(pathname.as_ptr().cast(), self.flags(), mode) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(File::<A> {
                _access: PhantomData,
                fd: Fd(fd),
            }),
        }
    }

    pub fn open_rel<P: AsRef<Path<Rel>>>(
        &self,
        relative_to: &Directory,
        file_path: P
    ) -> Result<File<A>, RawOsError> {
        let pathname = CString::from(file_path.as_ref().to_owned());
        let mode = self.mode.unwrap_or(0o644) as c_int;

        match unsafe { libc::openat(
            *relative_to.fd,
            // Skip the leading '/' so that the path is considered relative.
            pathname.as_ptr().add(1).cast(),
            self.flags(),
            mode
        ) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(File::<A> {
                _access: PhantomData,
                fd: Fd(fd),
            }),
        }
    }

    pub fn open_dir_entry(&self, dir_ent: &DirEntry) -> Result<File<A>, RawOsError> {
        self.open_rel(dir_ent.parent, &dir_ent.path)
    }

    pub const fn no_create(self) -> OpenOptions<A, NoCreate> {
        OpenOptions::<A, NoCreate> {
            _open: PhantomData,
            ..self
        }
    }

    pub const fn create_if_missing(self) -> OpenOptions<A, CreateIfMissing> {
        OpenOptions::<A, CreateIfMissing> {
            _open: PhantomData,
            ..self
        }
    }

    pub const fn create_or_empty(self) -> OpenOptions<A, CreateOrEmpty> {
        OpenOptions::<A, CreateOrEmpty> {
            _open: PhantomData,
            ..self
        }
    }

    pub const fn create(self) -> OpenOptions<A, Create> {
        OpenOptions::<A, Create> {
            _open: PhantomData,
            ..self
        }
    }

    pub const fn read_only(self) -> OpenOptions<ReadOnly, O> {
        OpenOptions::<ReadOnly, O> {
            _access: PhantomData,
            ..self
        }
    }

    pub const fn write_only(self) -> OpenOptions<WriteOnly, O> {
        OpenOptions::<WriteOnly, O> {
            _access: PhantomData,
            ..self
        }
    }

    pub const fn read_write(self) -> OpenOptions<ReadWrite, O> {
        OpenOptions::<ReadWrite, O> {
            _access: PhantomData,
            ..self
        }
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
impl<A: AccessMode, O: OpenMode> Default for OpenOptions<A, O> {
    fn default() -> Self {
        Self {
            _access: Default::default(),
            _open: Default::default(),
            mode: Default::default(),
            append: Default::default(),
            force_sync: Default::default(),
            update_access_time: Default::default(),
            follow_links: Default::default(),
            extra_flags: Default::default()
        }
    }
}

impl<A: AccessMode, O: OpenMode> Debug for OpenOptions<A, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenOptions")
            .field("<access>", &util::fmt::raw_type_name::<A>())
            .field("<open>", &util::fmt::raw_type_name::<O>())
            .field("mode", &self.mode)
            .field("append", &self.append)
            .field("force_sync", &self.force_sync)
            .field("update_access_time", &self.update_access_time)
            .field("follow_links", &self.follow_links)
            .field("extra_flags", &self.extra_flags)
            .finish()
    }
}