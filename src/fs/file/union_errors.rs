use derive_more::{Display, Error, From};

use super::{File, IOError, InterruptError, StorageExhaustedError, SyncUnsupportedError};

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

impl From<CloseError> for SyncError {
    fn from(value: CloseError) -> Self {
        match value {
            CloseError::Interrupt(e) => e.into(),
            CloseError::IO(e) => e.into(),
            CloseError::StorageExhausted(e) => e.into(),
        }
    }
}

pub struct SyncCloseError {
    pub error: SyncError,
    pub recovered: Option<File>,
}