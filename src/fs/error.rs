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
#[display("file metadata would overflow capacity")]
pub struct MetadataOverflowError;

#[derive(Debug, Display, Error)]
#[display("out of memory")]
pub struct OOMError;

#[derive(Debug, Display, Error)]
#[display("exceeded open file limit for process")]
pub struct FileCountError;

#[derive(Debug, Display, Error)]
#[display("exceeded memory for allocating locks")]
pub struct LockMemError;

#[derive(Debug, Display, Error)]
#[display("operating would block but called via non-blocking method")]
pub struct WouldBlockError;

#[derive(Debug, Display, Error)]
#[display("directory no longer exists")]
pub struct RemovedDirectoryError;

#[derive(Debug, Display, Error)]
#[display("search permission is denied for one of the directories in the provided path")]
pub struct NoSearchError;

#[derive(Debug, Display, Error)]
#[display("path contains too many symlinks")]
pub struct ExcessiveLinksError;

#[derive(Debug, Display, Error)]
#[display("path is too long")]
pub struct PathLengthError;

#[derive(Debug, Display, Error)]
#[display("a component of the provide path does not exist")]
pub struct MissingComponentError;

#[derive(Debug, Display, Error)]
#[display("a component of the provide path is not a directory")]
pub struct NonDirComponentError;
