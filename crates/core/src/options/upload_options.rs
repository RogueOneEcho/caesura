use crate::commands::CommandArguments::{self, *};
use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for upload
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct UploadOptions {
    /// Should the transcoded files be copied to the content directory?
    #[arg(long)]
    pub copy_transcode_to_content_dir: bool,

    /// Directory the transcoded files are copied to.
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    #[arg(long)]
    pub copy_transcode_to: Option<PathBuf>,

    /// Directory the torrent file is copied to.
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    #[arg(long)]
    pub copy_torrent_to: Option<PathBuf>,

    /// Is this a dry run?
    ///
    /// If enabled data won't be uploaded and will instead be printed to the console.
    #[arg(long)]
    pub dry_run: bool,
}

impl OptionsContract for UploadOptions {
    type Partial = UploadOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(dir) = &self.copy_transcode_to
            && !dir.is_dir()
        {
            errors.push(DoesNotExist(
                "Copy transcode to directory".to_owned(),
                dir.to_string_lossy().to_string(),
            ));
        }
        if let Some(dir) = &self.copy_torrent_to
            && !dir.is_dir()
        {
            errors.push(DoesNotExist(
                "Copy torrent to directory".to_owned(),
                dir.to_string_lossy().to_string(),
            ));
        }
    }
}

impl FromArgs for UploadOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { upload, .. } | Upload { upload, .. }) => Some(upload.clone()),
            _ => None,
        }
    }
}
