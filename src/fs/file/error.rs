use derive_more::{Display, Error, From};

use crate::fs::error::*;

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
    OversizedFile(OversizedFileError),
    Interrupt(InterruptError),
    InvalidBasename(InvalidBasenameError),
    IrregularFile(IrregularFileError),
    ExcessiveLinks(ExcessiveLinksError),
    FileCount(FileCountError),
    PathLength(PathLengthError),
    MetadataOverflow(MetadataOverflowError),
    MissingComponent(MissingComponentError),
    OOM(OOMError),
    StorageExhausted(StorageExhaustedError), 
    NonDirComponent(NonDirComponentError),
    Permission(PermissionError),
    ReadOnlyFS(ReadOnlyFSError),
    BusyExecutable(BusyExecutableError),
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
    // EACCES
    Access(AccessError),
    // EBUSY - Is this applicable? O_EXCL is only used with O_CREAT.
    // EDQUOTE
    StorageExhausted(StorageExhaustedError),
    // EEXIST
    AlreadyExists(AlreadyExistsError),
    // EFAULT -> Panic
    // EFBIG -> TODO
    // EINTR
    Interrupt(InterruptError),
}