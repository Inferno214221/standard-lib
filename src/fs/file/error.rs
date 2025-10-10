use std::io::RawOsError;

use derive_more::{Display, Error, From};

use libc::EOPNOTSUPP;
use libc::{EACCES, EBADF, EBUSY, EDQUOT, EEXIST, EFAULT, EFBIG, EINTR, EINVAL, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENXIO, EOVERFLOW, EPERM, EROFS, ETXTBSY};

use crate::fs::error::*;
use crate::fs::panic::*;

#[derive(Debug, Display, Clone, From, Error)]
pub enum CloseError {
    Interrupt(InterruptError),
    IO(IOError),
    StorageExhausted(StorageExhaustedError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum SyncError {
    Interrupt(InterruptError),
    IO(IOError),
    StorageExhausted(StorageExhaustedError),
    SyncUnsupported(SyncUnsupportedError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum CloneError {
    FileCount(FileCountError),
    OOM(OOMError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum LockError {
    Interrupt(InterruptError),
    LockMem(LockMemError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum TryLockError {
    Interrupt(InterruptError),
    LockMem(LockMemError),
    WouldBlock(WouldBlockError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum MetadataError {
    OOM(OOMError),
    MetadataOverflow(MetadataOverflowError),
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum FileTypeError {
    OOM(OOMError),
    IrregularFile(IrregularFileError),
    MetadataOverflow(MetadataOverflowError),
}

impl From<MetadataError> for FileTypeError {
    fn from(value: MetadataError) -> Self {
        match value {
            MetadataError::OOM(e) => e.into(),
            MetadataError::MetadataOverflow(e) => e.into(),
        }
    }
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum OpenError {
    Access(AccessError),
    Interrupt(InterruptError),
    StorageExhausted(StorageExhaustedError),
    OversizedFile(OversizedFileError),
    IrregularFile(IrregularFileError),
    InvalidBasename(InvalidBasenameError),
    ExcessiveLinks(ExcessiveLinksError),
    FileCount(FileCountError),
    PathLength(PathLengthError),
    MetadataOverflow(MetadataOverflowError),
    MissingComponent(MissingComponentError),
    OOM(OOMError),
    NonDirComponent(NonDirComponentError),
    Permission(PermissionError),
    ReadOnlyFS(ReadOnlyFSError),
    BusyExecutable(BusyExecutableError),
}

impl OpenError {
    pub(crate) fn interpret_raw_error(error: RawOsError) -> Self {
        match error {
            EACCES                          => AccessError.into(),
            EBADF                           => BadFdPanic.panic(),
            EBUSY | EISDIR | ENODEV | ENXIO => IrregularFileError.into(),
            EDQUOT | ENOSPC                 => StorageExhaustedError.into(),
            EFAULT                          => BadStackAddrPanic.panic(),
            EFBIG | EOVERFLOW               => OversizedFileError.into(),
            EINTR                           => InterruptError.into(),
            EINVAL                          => InvalidBasenameError.into(),
            ELOOP                           => ExcessiveLinksError.into(),
            EMFILE | ENFILE                 => FileCountError.into(),
            ENAMETOOLONG                    => PathLengthError.into(),
            ENOENT                          => MissingComponentError.into(),
            ENOMEM                          => OOMError.into(),
            ENOTDIR                         => NonDirComponentError.into(),
            EPERM                           => PermissionError.into(),
            EROFS                           => ReadOnlyFSError.into(),
            ETXTBSY                         => BusyExecutableError.into(),
            e                               => UnexpectedErrorPanic(e).panic(),
        }
    }
}

impl From<FileTypeError> for OpenError {
    fn from(value: FileTypeError) -> Self {
        match value {
            FileTypeError::OOM(e) => e.into(),
            FileTypeError::IrregularFile(e) => e.into(),
            FileTypeError::MetadataOverflow(e) => e.into(),
        }
    }
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum CreateError {
    Access(AccessError),
    Interrupt(InterruptError),
    StorageExhausted(StorageExhaustedError),
    OversizedFile(OversizedFileError),
    IrregularFile(IrregularFileError),
    InvalidBasename(InvalidBasenameError),
    ExcessiveLinks(ExcessiveLinksError),
    FileCount(FileCountError),
    PathLength(PathLengthError),
    MetadataOverflow(MetadataOverflowError),
    MissingComponent(MissingComponentError),
    OOM(OOMError),
    NonDirComponent(NonDirComponentError),
    Permission(PermissionError),
    ReadOnlyFS(ReadOnlyFSError),
    BusyExecutable(BusyExecutableError),
    AlreadyExists(AlreadyExistsError),
}

impl CreateError {
    pub(crate) fn interpret_raw_error(error: RawOsError) -> Self {
        match error {
            EACCES                          => AccessError.into(),
            EBADF                           => BadFdPanic.panic(),
            EBUSY | EISDIR | ENODEV | ENXIO => IrregularFileError.into(),
            EDQUOT | ENOSPC                 => StorageExhaustedError.into(),
            EEXIST                          => AlreadyExistsError.into(),
            EFAULT                          => BadStackAddrPanic.panic(),
            EFBIG | EOVERFLOW               => OversizedFileError.into(),
            EINTR                           => InterruptError.into(),
            EINVAL                          => InvalidBasenameError.into(),
            ELOOP                           => ExcessiveLinksError.into(),
            EMFILE | ENFILE                 => FileCountError.into(),
            ENAMETOOLONG                    => PathLengthError.into(),
            ENOENT                          => MissingComponentError.into(),
            ENOMEM                          => OOMError.into(),
            ENOTDIR                         => NonDirComponentError.into(),
            EPERM                           => PermissionError.into(),
            EROFS                           => ReadOnlyFSError.into(),
            ETXTBSY                         => BusyExecutableError.into(),
            e                               => UnexpectedErrorPanic(e).panic(),
        }
    }
}

impl From<FileTypeError> for CreateError {
    fn from(value: FileTypeError) -> Self {
        match value {
            FileTypeError::OOM(e) => e.into(),
            FileTypeError::IrregularFile(e) => e.into(),
            FileTypeError::MetadataOverflow(e) => e.into(),
        }
    }
}

#[derive(Debug, Display, Clone, From, Error)]
pub enum TempError {
    Access(AccessError),
    Interrupt(InterruptError),
    StorageExhausted(StorageExhaustedError),
    OversizedFile(OversizedFileError),
    IrregularFile(IrregularFileError),
    InvalidBasename(InvalidBasenameError),
    ExcessiveLinks(ExcessiveLinksError),
    FileCount(FileCountError),
    PathLength(PathLengthError),
    MetadataOverflow(MetadataOverflowError),
    MissingComponent(MissingComponentError),
    OOM(OOMError),
    NonDirComponent(NonDirComponentError),
    Permission(PermissionError),
    ReadOnlyFS(ReadOnlyFSError),
    BusyExecutable(BusyExecutableError),
    TempFileUnsupported(TempFileUnsupportedError),
}

impl TempError {
    pub(crate) fn interpret_raw_error(error: RawOsError) -> Self {
        match error {
            EACCES                 => AccessError.into(),
            EBADF                  => BadFdPanic.panic(),
            EBUSY | ENODEV | ENXIO => IrregularFileError.into(),
            EDQUOT | ENOSPC        => StorageExhaustedError.into(),
            EFAULT                 => BadStackAddrPanic.panic(),
            EFBIG | EOVERFLOW      => OversizedFileError.into(),
            EINTR                  => InterruptError.into(),
            EINVAL                 => InvalidBasenameError.into(),
            EISDIR | EOPNOTSUPP    => TempFileUnsupportedError.into(),
            ELOOP                  => ExcessiveLinksError.into(),
            EMFILE | ENFILE        => FileCountError.into(),
            ENAMETOOLONG           => PathLengthError.into(),
            // TODO: Has an additional interpretation for temp files.
            ENOENT                 => MissingComponentError.into(),
            ENOMEM                 => OOMError.into(),
            ENOTDIR                => NonDirComponentError.into(),
            EPERM                  => PermissionError.into(),
            EROFS                  => ReadOnlyFSError.into(),
            ETXTBSY                => BusyExecutableError.into(),
            e                      => UnexpectedErrorPanic(e).panic(),
        }
    }
}

impl From<FileTypeError> for TempError {
    fn from(value: FileTypeError) -> Self {
        match value {
            FileTypeError::OOM(e) => e.into(),
            FileTypeError::IrregularFile(e) => e.into(),
            FileTypeError::MetadataOverflow(e) => e.into(),
        }
    }
}