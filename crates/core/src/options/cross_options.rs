use crate::prelude::*;

/// Options for the `cross` command behavior.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct CrossOptions {
    /// Perform the cross seed lookup but skip downloading and injection.
    #[arg(long)]
    pub dry_run: bool,

    /// Directory the cross-seed `.torrent` file is copied to after download.
    ///
    /// This should be set if you wish to auto-add to your torrent client via a watch directory.
    #[arg(long)]
    pub copy_cross_torrent_to: Option<PathBuf>,
}

impl OptionsContract for CrossOptions {
    type Partial = CrossOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(dir) = &self.copy_cross_torrent_to
            && !dir.is_dir()
        {
            errors.push(OptionRule::DoesNotExist(
                "Copy cross torrent to directory".to_owned(),
                dir.to_string_lossy().to_string(),
            ));
        }
    }
}
