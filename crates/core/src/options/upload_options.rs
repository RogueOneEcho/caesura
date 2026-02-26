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

    /// Rsync destination for transcoded files.
    ///
    /// Supports local destinations and SSH destinations such as `user@host:/path`.
    #[arg(long)]
    pub rsync_transcode_to: Option<String>,

    /// Rsync destination for torrent files.
    ///
    /// Supports local destinations and SSH destinations such as `user@host:/path`.
    #[arg(long)]
    pub rsync_torrent_to: Option<String>,

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
