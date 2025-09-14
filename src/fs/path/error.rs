use derive_more::{Display, Error, From};

use crate::fs::error::{EmptyStrError, ExcessiveLinksError, HomeResolutionError, MissingComponentError, NoSearchError, NonDirComponentError, PathLengthError};
use crate::fs::file::MetadataError;

#[derive(Debug, Display, Clone, From, Error)]
pub enum PathError {
    NoSearch(NoSearchError),
    ExcessiveLinks(ExcessiveLinksError),
    PathLength(PathLengthError),
    MissingComponent(MissingComponentError),
    NonDirComponent(NonDirComponentError),
}

#[derive(Debug, Display, Clone, From, Error)]
// TODO: Maybe make this a union type?
pub enum PathOrMetadataError {
    Path(PathError),
    Metadata(MetadataError),
}


#[derive(Debug, Display, Clone, From, Error)]
pub enum PathParseError {
    EmptyStr(EmptyStrError),
    HomeResolution(HomeResolutionError),
}