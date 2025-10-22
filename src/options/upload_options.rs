use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::{ArgAction, Args};
use di::{Ref, injectable};
use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::*;
use crate::commands::*;
use crate::options::*;

/// Options for including additional files during [`TranscodeCommand`]
#[derive(Args, Clone, Debug, Default, Deserialize, Serialize)]
pub struct UploadOptions {
    /// Should the transcoded files be copied to the content directory?
    ///
    /// Default: `false`
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub copy_transcode_to_content_dir: Option<bool>,

    /// Directory the transcoded files are copied to.
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    ///
    /// Default: Not set
    #[arg(long)]
    pub copy_transcode_to: Option<PathBuf>,

    /// Directory the torrent file is copied to.
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    ///
    /// Default: Not set
    #[arg(long)]
    pub copy_torrent_to: Option<PathBuf>,

    /// Is this a dry run?
    ///
    /// If enabled data won't be uploaded and will instead be printed to the console.
    ///
    /// Default: `false`
    #[arg(long, default_value = None, action = ArgAction::SetTrue)]
    pub dry_run: Option<bool>,
}

#[injectable]
impl UploadOptions {
    fn new(provider: Ref<OptionsProvider>) -> Self {
        provider.get()
    }
}

impl Options for UploadOptions {
    fn merge(&mut self, alternative: &Self) {
        if self.copy_transcode_to_content_dir.is_none() {
            self.copy_transcode_to_content_dir = alternative.copy_transcode_to_content_dir;
        }
        if self.copy_transcode_to.is_none() {
            self.copy_transcode_to
                .clone_from(&alternative.copy_transcode_to);
        }
        if self.copy_torrent_to.is_none() {
            self.copy_torrent_to
                .clone_from(&alternative.copy_torrent_to);
        }
        if self.dry_run.is_none() {
            self.dry_run = alternative.dry_run;
        }
    }

    fn apply_defaults(&mut self) {
        if self.copy_transcode_to_content_dir.is_none() {
            self.copy_transcode_to_content_dir = Some(false);
        }
        if self.dry_run.is_none() {
            self.dry_run = Some(false);
        }
    }

    fn validate(&self) -> bool {
        let mut errors: Vec<OptionRule> = Vec::new();
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
        OptionRule::show(&errors);
        errors.is_empty()
    }

    fn from_args() -> Option<Self> {
        let Some(Upload { upload, .. } | Batch { upload, .. }) = ArgumentsParser::get() else {
            return None;
        };
        let mut options = upload;
        if options.copy_transcode_to_content_dir == Some(false) {
            options.copy_transcode_to_content_dir = None;
        }
        if options.dry_run == Some(false) {
            options.dry_run = None;
        }
        Some(options)
    }

    fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Display for UploadOptions {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let output = if let Ok(yaml) = serde_yaml::to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        output.fmt(formatter)
    }
}
