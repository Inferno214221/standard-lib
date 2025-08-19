use std::io::RawOsError;

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display("interrupted by signal")]
pub struct InterruptError;

#[derive(Debug, Display, Error)]
#[display("error during I/O")]
pub struct IOError;

#[derive(Debug, Display, Error)]
#[display("available storage space exhausted")]
pub struct StorageExhaustedError;

#[derive(Debug, Display, Error)]
#[display("sync not supported by file")]
pub struct SyncUnsupportedError;

#[derive(Debug, Display, Error)]
#[display("file descriptor corruption")]
pub struct BadFDError;

#[derive(Debug, Display, Error)]
#[display("pointer exceeded stack space")]
pub struct BadStackAddrError;

#[derive(Debug, Display, Error)]
#[display("file metadata would overflow capacity")]
pub struct FileMetaOverflowError;

#[derive(Debug, Display, Error)]
#[display("out of memory")]
pub struct OOMError;

#[derive(Debug, Display, Error)]
#[display("unexpected OS error with code: {_0}")]
pub struct UnexpectedError(#[error(not(source))] pub RawOsError);
