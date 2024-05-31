use crate::verify::SourceRule::*;
use std::fmt::{Display, Formatter};

pub enum SourceRule {
    SceneNotSupported,
    LossyMasterNeedsApproval,
    LossyWebNeedsApproval,
    NoTranscodeFormats,
    SourceDirectoryNotFound(String),
    NoFlacFiles(String),
    IncorrectHash,
    NoArtistTag(String),
    NoAlbumTag(String),
    NoTitleTag(String),
    NoTrackNumberTag(String),
}

impl Display for SourceRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            SceneNotSupported => "Scene releases are not supported".to_owned(),
            LossyMasterNeedsApproval => "Lossy master releases need approval".to_owned(),
            LossyWebNeedsApproval => "Lossy web releases need approval".to_owned(),
            NoTranscodeFormats => "All allowed formats have been transcoded to already".to_owned(),
            SourceDirectoryNotFound(_) => "Source directory not found: {0}".to_owned(),
            NoFlacFiles(path) => format!("No Flac files found in source directory: {path}"),
            IncorrectHash => "Files do not match hash".to_owned(),
            NoArtistTag(path) => format!("No artist tag: {path}"),
            NoAlbumTag(path) => format!("No album tag: {path}"),
            NoTitleTag(path) => format!("No title tag: {path}"),
            NoTrackNumberTag(path) => format!("No track number tag: {path}"),
        };
        message.fmt(formatter)
    }
}
