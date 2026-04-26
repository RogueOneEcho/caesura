use crate::prelude::*;

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

    fn validate(&self, validator: &mut OptionsValidator) {
        if let Some(dir) = &self.copy_transcode_to {
            validator.check_dir_exists("copy_transcode_to", dir);
        }
        if let Some(dir) = &self.copy_torrent_to {
            validator.check_dir_exists("copy_torrent_to", dir);
        }
    }
}
