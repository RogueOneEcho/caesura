use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for injecting generated torrents into a torrent client.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct TorrentInjectionOptions {
    /// Directory the torrent file is copied to.
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    #[arg(long)]
    pub copy_torrent_to: Option<PathBuf>,
}

impl OptionsContract for TorrentInjectionOptions {
    type Partial = TorrentInjectionOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
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
