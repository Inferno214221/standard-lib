use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use libc::{O_APPEND, O_CREAT, O_DIRECTORY, O_EXCL, O_NOATIME, O_NOFOLLOW, O_SYNC, O_TMPFILE, O_TRUNC, c_int};

use super::{AccessMode, DEF_FILE_MODE, File};
use crate::fs::dir::DirEntry;
use crate::fs::file::{Create, CreateError, CreateIfMissing, CreateOrEmpty, CreateTemp, CreateUnlinked, NoCreate, OpenError, OpenMode, Permanent, ReadOnly, ReadWrite, TempError, Temporary, Write, WriteOnly};
use crate::fs::{Abs, Directory, Fd, FileType, OwnedPath, Path, Rel};
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

macro_rules! impl_open {
    ($mode:ty) => {
        impl<A: AccessMode> OpenOptions<A, $mode> {
            pub fn open<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<File<A>, OpenError> {
                match Fd::open(file_path, self.flags(), self.mode) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd: fd.assert_type(FileType::Regular)?,
                    }),
                    Err(e) => Err(OpenError::interpret_raw_error(e)),
                }
            }

            pub fn open_rel<P: AsRef<Path<Rel>>>(
                &self,
                relative_to: &Directory,
                file_path: P
            ) -> Result<File<A>, OpenError> {
                match Fd::open_rel(relative_to, file_path, self.flags(), self.mode) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd: fd.assert_type(FileType::Regular)?,
                    }),
                    Err(e) => Err(OpenError::interpret_raw_error(e)),
                }
            }

            pub fn open_dir_entry(&self, dir_ent: &DirEntry) -> Result<File<A>, OpenError> {
                self.open_rel(dir_ent.parent, &dir_ent.path)
            }
        }
    };
    ($mode:ty, $($a:ty),+) => {
        impl_open!($mode);
        impl_open!($($a),+);
    };
}

macro_rules! impl_create {
    ($mode:ty) => {
        impl<A: AccessMode> OpenOptions<A, $mode> {
            pub fn open<P: AsRef<Path<Abs>>>(&self, file_path: P) -> Result<File<A>, CreateError> {
                match Fd::open(file_path, self.flags(), self.mode) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd,
                    }),
                    Err(e) => Err(CreateError::interpret_raw_error(e)),
                }
            }

            pub fn open_rel<P: AsRef<Path<Rel>>>(
                &self,
                relative_to: &Directory,
                file_path: P
            ) -> Result<File<A>, CreateError> {
                match Fd::open_rel(relative_to, file_path, self.flags(), self.mode) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd,
                    }),
                    Err(e) => Err(CreateError::interpret_raw_error(e)),
                }
            }
        }
    };
    ($mode:ty, $($a:ty),+) => {
        impl_create!($mode);
        impl_create!($($a),+);
    };
}

macro_rules! impl_create_temp {
    ($mode:ty) => {
        impl<A: AccessMode> OpenOptions<A, $mode> {
            pub fn open<P: AsRef<Path<Abs>>>(&self, dir_path: P) -> Result<File<A>, TempError> {
                match Fd::open(dir_path, self.flags(), self.mode) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd,
                    }),
                    Err(e) => Err(TempError::interpret_raw_error(e)),
                }
            }

            pub fn open_rel(
                &self,
                relative_to: &Directory
            ) -> Result<File<A>, TempError> {
                match Fd::open_rel(
                    relative_to,
                    OwnedPath::dot_slash_dot(),
                    self.flags(),
                    self.mode
                ) {
                    Ok(fd) => Ok(File::<A> {
                        _access: PhantomData,
                        fd,
                    }),
                    Err(e) => Err(TempError::interpret_raw_error(e)),
                }
            }
        }
    };
    ($mode:ty, $($a:ty),+) => {
        impl_create_temp!($mode);
        impl_create_temp!($($a),+);
    };
}

impl_open! {
    NoCreate,
    CreateIfMissing,
    CreateOrEmpty
}

impl_create! {
    Create
}

impl_create_temp! {
    CreateTemp,
    CreateUnlinked
}

// The Default derive macro doesn't like my spooky zero-variant enums.
impl<A: AccessMode, O: OpenMode> Default for OpenOptions<A, O> {
    fn default() -> Self {
        Self {
            _access: Default::default(),
            _open: Default::default(),
            mode: DEF_FILE_MODE,
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