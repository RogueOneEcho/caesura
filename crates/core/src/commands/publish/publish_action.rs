use crate::prelude::*;

/// Actions that can fail in the publish module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum PublishAction {
    #[error("validate tracker")]
    ValidateTracker,
    #[error("parse manifest")]
    ParseManifest,
    #[error("validate manifest")]
    ValidateManifest,
    #[error("create source torrent")]
    CreateTorrent,
    #[error("stage source for seeding")]
    StageSource,
    #[error("verify seeding content")]
    VerifySeedContent,
    #[error("inject torrent into client")]
    InjectTorrent,
    #[error("get torrent group")]
    GetTorrentGroup,
    #[error("check duplicate source")]
    CheckDuplicate,
    #[error("upload new source")]
    UploadNewSource,
    #[error("upload source to existing group")]
    UploadExistingGroup,
}

/// Errors that can occur during publish.
#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
pub enum PublishError {
    #[error("publish is currently RED-only, received indexer '{indexer}'")]
    UnsupportedIndexer { indexer: String },
    #[error("existing-group source upload duplicates an existing format for this release")]
    DuplicateSource,
    #[error("source format must be FLAC/Lossless or FLAC/24bit Lossless")]
    UnsupportedSourceFormat,
    #[error("staged content failed torrent verification: {issue}")]
    SeedContentVerification { issue: String },
}
