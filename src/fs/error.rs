use derive_more::{Display, Error};

#[derive(Debug, Display, Clone, Error)]
#[display("interrupted by signal")]
pub struct InterruptError;

#[derive(Debug, Display, Clone, Error)]
#[display("error during I/O")]
pub struct IOError;

#[derive(Debug, Display, Clone, Error)]
#[display("available storage space exhausted")]
pub struct StorageExhaustedError;

#[derive(Debug, Display, Clone, Error)]
#[display("sync not supported by file")]
pub struct SyncUnsupportedError;

#[derive(Debug, Display, Clone, Error)]
#[display("temp file not supported by filesystem")]
pub struct TempFileUnsupportedError;

#[derive(Debug, Display, Clone, Error)]
#[display("file metadata would overflow capacity")]
pub struct MetadataOverflowError;

#[derive(Debug, Display, Clone, Error)]
#[display("out of memory")]
pub struct OOMError;

#[derive(Debug, Display, Clone, Error)]
#[display("exceeded open file limit for process")]
pub struct FileCountError;

#[derive(Debug, Display, Clone, Error)]
#[display("exceeded memory for allocating locks")]
pub struct LockMemError;

#[derive(Debug, Display, Clone, Error)]
#[display("operation would block but called via non-blocking method")]
pub struct WouldBlockError;

#[derive(Debug, Display, Clone, Error)]
#[display("directory no longer exists")]
pub struct RemovedDirectoryError;

#[derive(Debug, Display, Clone, Error)]
#[display("search permission is denied for one of the directories in the provided path")]
pub struct NoSearchError;

#[derive(Debug, Display, Clone, Error)]
#[display("path contains too many symlinks")]
pub struct ExcessiveLinksError;

#[derive(Debug, Display, Clone, Error)]
#[display("path is too long")]
pub struct PathLengthError;

#[derive(Debug, Display, Clone, Error)]
#[display("a component of the provided path does not exist")]
pub struct MissingComponentError;

#[derive(Debug, Display, Clone, Error)]
#[display("a component of the provided path is not a directory")]
pub struct NonDirComponentError;

#[derive(Debug, Display, Clone, Error)]
#[display("provided string is empty")]
pub struct EmptyStrError;

#[derive(Debug, Display, Clone, Error)]
#[display("unable to get home directory")]
pub struct HomeResolutionError;

#[derive(Debug, Display, Clone, Error)]
#[display("access to file system component denied")]
pub struct AccessError;

#[derive(Debug, Display, Clone, Error)]
#[display("provided path already exists")]
pub struct AlreadyExistsError;

#[derive(Debug, Display, Clone, Error)]
#[display("file is too large to open")]
pub struct OversizedFileError;

#[derive(Debug, Display, Clone, Error)]
#[display("operation was prevented by a file seal or lack or privileges to avoid updating access time")]
pub struct PermissionError;

#[derive(Debug, Display, Clone, Error)]
#[display("write access requested for file on a read-only filesystem")]
pub struct ReadOnlyFSError;

#[derive(Debug, Display, Clone, Error)]
#[display("the basename of the provided path is not permitted by the filesystem")]
pub struct InvalidBasenameError;

#[derive(Debug, Display, Clone, Error)]
#[display("write access was requested on a file that is currently being executed or used by the kernel in some way")]
pub struct BusyExecutableError;

#[derive(Debug, Display, Clone, Error)]
#[display("provided path refers to a file of the wrong type")]
pub struct IncorrectTypeError;