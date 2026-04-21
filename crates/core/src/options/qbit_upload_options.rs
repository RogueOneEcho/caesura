use crate::prelude::*;
use qbittorrent_api::add_torrent::AddTorrentOptions;

/// Options controlling qBittorrent torrent injection on upload.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QbitUploadOptions {
    /// Should the torrent be injected into qBittorrent after upload?
    #[arg(long)]
    pub inject_torrent: bool,

    /// qBittorrent save path for injected torrents.
    #[arg(long)]
    pub qbit_inject_savepath: Option<String>,

    /// qBittorrent category for injected torrents.
    #[arg(long)]
    pub qbit_inject_category: Option<String>,

    /// qBittorrent tags for injected torrents.
    #[arg(long)]
    pub qbit_inject_tags: Option<Vec<String>>,

    /// Add injected torrents in paused state.
    #[arg(long)]
    pub qbit_inject_paused: Option<bool>,

    /// Skip hash checking when injecting torrents.
    #[arg(long)]
    pub qbit_inject_skip_checking: Option<bool>,
}

impl QbitUploadOptions {
    /// Build `AddTorrentOptions` for the qBittorrent API from these options.
    #[must_use]
    pub fn to_add_torrent_options(&self) -> AddTorrentOptions {
        AddTorrentOptions {
            save_path: self.qbit_inject_savepath.clone(),
            category: self.qbit_inject_category.clone(),
            tags: self.qbit_inject_tags.clone(),
            paused: self.qbit_inject_paused,
            skip_checking: self.qbit_inject_skip_checking,
            ..AddTorrentOptions::default()
        }
    }

    /// Create a [`QbitUploadOptions`] with mock values for testing.
    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            inject_torrent: true,
            qbit_inject_savepath: None,
            qbit_inject_category: Some(APP_NAME.to_owned()),
            qbit_inject_tags: Some(vec![APP_NAME.to_owned()]),
            qbit_inject_paused: None,
            qbit_inject_skip_checking: None,
        }
    }
}

impl OptionsContract for QbitUploadOptions {
    type Partial = QbitUploadOptionsPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
