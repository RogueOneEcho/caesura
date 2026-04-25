use crate::prelude::*;
use qbittorrent_api::add_torrent::AddTorrentOptions;

/// qBittorrent injection options for the `cross` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QbitCrossOptions {
    /// Should the cross-seed torrent be injected into qBittorrent?
    #[arg(long)]
    pub qbit_cross: bool,

    /// qBittorrent save path for cross-seed torrents.
    #[arg(long)]
    pub qbit_cross_savepath: Option<String>,

    /// qBittorrent category for cross-seed torrents.
    ///
    /// Default: `caesura`
    #[arg(long)]
    pub qbit_cross_category: Option<String>,

    /// qBittorrent tags for cross-seed torrents.
    ///
    /// Default: `["caesura"]`
    #[arg(long)]
    pub qbit_cross_tags: Option<Vec<String>>,

    /// Add cross-seed torrents in paused state.
    #[arg(long)]
    pub qbit_cross_paused: Option<bool>,

    /// Skip hash checking when injecting cross-seed torrents.
    #[arg(long)]
    pub qbit_cross_skip_checking: Option<bool>,
}

impl QbitCrossOptions {
    /// Build [`AddTorrentOptions`] for the qBittorrent API from these options.
    #[must_use]
    pub fn to_add_torrent_options(&self) -> AddTorrentOptions {
        AddTorrentOptions {
            save_path: self.qbit_cross_savepath.clone(),
            category: self.qbit_cross_category.clone(),
            tags: self.qbit_cross_tags.clone(),
            paused: self.qbit_cross_paused,
            skip_checking: self.qbit_cross_skip_checking,
            ..AddTorrentOptions::default()
        }
    }

    /// Create a [`QbitCrossOptions`] with mock values for testing.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            qbit_cross: true,
            qbit_cross_savepath: None,
            qbit_cross_category: Some(APP_NAME.to_owned()),
            qbit_cross_tags: Some(vec![APP_NAME.to_owned()]),
            qbit_cross_paused: None,
            qbit_cross_skip_checking: None,
        }
    }
}

impl OptionsContract for QbitCrossOptions {
    type Partial = QbitCrossOptionsPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
