use derive_more::{Display, Error, From};

use crate::fs::error::{ExcessiveLinksError, MissingComponentError, NoSearchError, NonDirComponentError, PathLengthError};
use crate::fs::file::MetadataError;

#[derive(Debug, Display, From, Error)]
pub enum PathError {
    NoSearch(NoSearchError),
    ExcessiveLinks(ExcessiveLinksError),
    PathLength(PathLengthError),
    MissingComponent(MissingComponentError),
    NonDirComponent(NonDirComponentError),
}

#[derive(Debug, Display, From, Error)]
// TODO: Maybe make this a union type?
pub enum PathOrMetadataError {
    Path(PathError),
    Metadata(MetadataError),
}
