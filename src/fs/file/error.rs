use derive_more::{Display, Error, From};

use crate::fs::error::{FileCountError, MetadataOverflowError, IOError, InterruptError, LockMemError, OOMError, StorageExhaustedError, SyncUnsupportedError, WouldBlockError};

#[derive(Debug, Display, From, Error)]
pub enum CloseError {
    Interrupt(InterruptError),
    IO(IOError),
    StorageExhausted(StorageExhaustedError),
}

#[derive(Debug, Display, From, Error)]
pub enum SyncError {
    Interrupt(InterruptError),
    IO(IOError),
    StorageExhausted(StorageExhaustedError),
    SyncUnsupported(SyncUnsupportedError),
}

#[derive(Debug, Display, From, Error)]
pub enum CloneError {
    FileCount(FileCountError),
    OOM(OOMError),
}

#[derive(Debug, Display, From, Error)]
pub enum LockError {
    Interrupt(InterruptError),
    LockMem(LockMemError),
}

#[derive(Debug, Display, From, Error)]
pub enum TryLockError {
    Interrupt(InterruptError),
    LockMem(LockMemError),
    WouldBlock(WouldBlockError),
}

#[derive(Debug, Display, From, Error)]
pub enum MetadataError {
    OOM(OOMError),
    MetadataOverflow(MetadataOverflowError),
}
