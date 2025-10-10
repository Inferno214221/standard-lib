use std::fmt;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::io::RawOsError;
use std::marker::PhantomData;

use libc::{EACCES, EFAULT, EFBIG, EINTR, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOVERFLOW, EPERM, EROFS, ETXTBSY, O_APPEND, O_CREAT, O_DIRECTORY, O_EXCL, O_NOATIME, O_NOFOLLOW, O_SYNC, O_TMPFILE, O_TRUNC, c_int};

use super::{File, AccessMode};
use crate::fs::dir::DirEntry;
use crate::fs::error::{AccessError, BusyExecutableError, ExcessiveLinksError, FileCountError, InterruptError, InvalidBasenameError, IrregularFileError, MissingComponentError, NonDirComponentError, OOMError, OversizedFileError, PathLengthError, PermissionError, ReadOnlyFSError, StorageExhaustedError};
use crate::fs::file::{Create, CreateIfMissing, CreateOrEmpty, CreateTemp, CreateUnlinked, NoCreate, OpenError, OpenMode, Permanent, ReadOnly, ReadWrite, Write, WriteOnly};
use crate::fs::panic::{BadStackAddrPanic, Panic, UnexpectedErrorPanic};
use crate::fs::path::{Abs, Path};
use crate::fs::{Directory, Fd, Rel};
use crate::util;
use crate::util::fmt::DebugRaw;

pub(crate) const EXTRA_FLAGS_MASK: c_int = !(
    O_APPEND | O_NOATIME | O_NOFOLLOW | O_SYNC | O_CREAT | O_EXCL | O_TRUNC | O_TMPFILE | O_DIRECTORY
);

/// A builder struct to help with opening files, using customizable options and logical defaults.
/// Available via [`File::options`] to avoid additional use statements.
// TODO: More docs here.
#[derive(Clone)]
pub struct OpenOptions<Access: AccessMode, Open: OpenMode> {
    pub(crate) _access: PhantomData<fn() -> Access>,
    pub(crate) _open: PhantomData<fn() -> Open>,
    pub(crate) flags: c_int,
    pub(crate) mode: c_int,
}

macro_rules! set_flag {
    ($self:ident, $value:expr, $flag:expr) => {
        if $value {
            $self.flags |= $flag;
        } else {
            $self.flags &= !$flag;
        }
    };
}

macro_rules! get_flag {
    ($self:ident, $flag:expr) => {
        $self.flags & $flag != 0
    };
}

impl<A: AccessMode, O: OpenMode> OpenOptions<A, O> {
    pub(crate) const fn flags(&self) -> c_int {
        self.flags | A::FLAGS | O::FLAGS
    }

    pub fn new() -> OpenOptions<A, O> {
        OpenOptions::<A, O>::default()
    }

    pub fn open_raw<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<Fd, RawOsError> {
        let pathname = CString::from(file_path.as_ref().to_owned());
        // TODO: Permission builder of some type?
        dbg!(&pathname);
        dbg!(&format!("{:x?}", self.flags()));

        match unsafe { libc::open(pathname.as_ptr().cast(), self.flags(), self.mode as c_int) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(Fd(fd)),
        }
    }

    pub fn open_rel_raw<P: AsRef<Path<Rel>>>(
        &self,
        relative_to: &Directory,
        file_path: P
    ) -> Result<Fd, RawOsError> {
        let pathname = CString::from(file_path.as_ref().to_owned());

        match unsafe { libc::openat(
            *relative_to.fd,
            // Skip the leading '/' so that the path is considered relative.
            pathname.as_ptr().add(1).cast(),
            self.flags(),
            self.mode
        ) } {
            -1 => Err(util::fs::err_no()),
            fd => Ok(Fd(fd)),
        }
    }

    pub fn open_dir_entry_raw(&self, dir_ent: &DirEntry) -> Result<Fd, RawOsError> {
        self.open_rel_raw(dir_ent.parent, &dir_ent.path)
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
        self.mode = value as c_int;
        self
    }

    pub const fn append(&mut self, value: bool) -> &mut Self {
        set_flag!(self, value, O_APPEND);
        self
    }

    pub const fn force_sync(&mut self, value: bool) -> &mut Self {
        set_flag!(self, value, O_SYNC);
        self
    }

    pub const fn update_access_time(&mut self, value: bool) -> &mut Self {
        set_flag!(self, !value, O_NOATIME);
        self
    }

    pub const fn follow_links(&mut self, value: bool) -> &mut Self {
        set_flag!(self, !value, O_NOFOLLOW);
        self
    }

    pub const unsafe fn extra_flags(&mut self, value: i32) -> &mut Self {
        self.flags |= value & EXTRA_FLAGS_MASK;
        self
    }
}

impl<A: AccessMode, O: Permanent> OpenOptions<A, O> {
    pub const fn read_only(self) -> OpenOptions<ReadOnly, O> {
        OpenOptions::<ReadOnly, O> {
            _access: PhantomData,
            ..self
        }
    }
}

impl<A: Write, O: OpenMode> OpenOptions<A, O> {
    pub const fn create_temp(self) -> OpenOptions<A, CreateTemp> {
        OpenOptions::<A, CreateTemp> {
            _open: PhantomData,
            ..self
        }
    }

    pub const fn create_unlinked(self) -> OpenOptions<A, CreateUnlinked> {
        OpenOptions::<A, CreateUnlinked> {
            _open: PhantomData,
            ..self
        }
    }
}

impl<A: AccessMode> OpenOptions<A, NoCreate> {
    pub fn open<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<File<A>, OpenError> {
        match self.open_raw(file_path) {
            Ok(fd) => Ok(File::<A> {
                _access: PhantomData,
                fd: fd.assert_type_reg()?,
            }),
            Err(e) => match e {
                EACCES                  => Err(AccessError)?,
                EFAULT                  => BadStackAddrPanic.panic(),
                EFBIG | EOVERFLOW       => Err(OversizedFileError)?,
                EINTR                   => Err(InterruptError)?,
                EINVAL                  => Err(InvalidBasenameError)?,
                EISDIR | ENODEV | ENXIO => Err(IrregularFileError)?,
                ELOOP                   => Err(ExcessiveLinksError)?,
                EMFILE | ENFILE         => Err(FileCountError)?,
                ENAMETOOLONG            => Err(PathLengthError)?,
                ENOENT                  => Err(MissingComponentError)?,
                ENOMEM                  => Err(OOMError)?,
                ENOSPC                  => Err(StorageExhaustedError)?, 
                ENOTDIR                 => Err(NonDirComponentError)?,
                EPERM                   => Err(PermissionError)?,
                EROFS                   => Err(ReadOnlyFSError)?,
                ETXTBSY                 => Err(BusyExecutableError)?,
                e                       => UnexpectedErrorPanic(e).panic(),
            },
        }
    }
}

// The Default derive macro doesn't like my spooky zero-variant enums.
impl<A: AccessMode, O: OpenMode> Default for OpenOptions<A, O> {
    fn default() -> Self {
        Self {
            _access: Default::default(),
            _open: Default::default(),
            mode: 0o644,
            flags: 0x0,
        }
    }
}

impl<A: AccessMode, O: OpenMode> Debug for OpenOptions<A, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenOptions")
            .field("<access>", &util::fmt::raw_type_name::<A>())
            .field("<open>", &util::fmt::raw_type_name::<O>())
            .field("mode", &DebugRaw(format!("0o{:o}", self.mode)))
            .field("append", &get_flag!(self, O_APPEND))
            .field("force_sync", &get_flag!(self, O_SYNC))
            .field("update_access_time", &!get_flag!(self, O_NOATIME))
            .field("follow_links", &!get_flag!(self, O_NOFOLLOW))
            .field("extra_flags", &DebugRaw(format!("0x{:x}", self.flags & EXTRA_FLAGS_MASK)))
            .finish()
    }
}