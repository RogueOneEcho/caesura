use crate::prelude::*;
use caesura_macros::Options;
use qbittorrent_api::add_torrent::AddTorrentOptions;
use serde::{Deserialize, Serialize};

/// Options for qBittorrent torrent injection
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QbitOptions {
    /// Should the torrent be injected into qBittorrent?
    ///
    /// This should be set if you wish to auto-add to your torrent client.
    #[arg(long)]
    pub inject_torrent: bool,

    /// qBittorrent API base URL.
    ///
    /// Example: `http://127.0.0.1:8080`
    #[arg(long)]
    pub qbit_url: Option<String>,

    /// qBittorrent username.
    #[arg(long)]
    pub qbit_username: Option<String>,

    /// qBittorrent password.
    #[arg(long)]
    pub qbit_password: Option<String>,

    /// qBittorrent save path for injected torrents.
    #[arg(long)]
    pub qbit_savepath: Option<String>,

    /// qBittorrent category for injected torrents.
    #[arg(long)]
    pub qbit_category: Option<String>,

    /// qBittorrent tags for injected torrents.
    #[arg(long)]
    pub qbit_tags: Option<Vec<String>>,

    /// Add injected torrents in paused state.
    #[arg(long)]
    pub qbit_paused: Option<bool>,

    /// Skip hash checking when injecting torrents.
    #[arg(long)]
    pub qbit_skip_checking: Option<bool>,
}

impl QbitOptions {
    #[must_use]
    pub fn to_add_torrent_options(&self) -> AddTorrentOptions {
        AddTorrentOptions {
            save_path: self.qbit_savepath.clone(),
            category: self.qbit_category.clone(),
            tags: self.qbit_tags.clone(),
            paused: self.qbit_paused,
            skip_checking: self.qbit_skip_checking,
            ..AddTorrentOptions::default()
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn mock() -> Self {
        Self {
            inject_torrent: true,
            qbit_url: Some("http://127.0.0.1:8080".to_owned()),
            qbit_username: Some("user".to_owned()),
            qbit_password: Some("hunter2".to_owned()),
            qbit_savepath: None,
            qbit_category: Some(APP_NAME.to_owned()),
            qbit_tags: Some(vec![APP_NAME.to_owned()]),
            qbit_paused: None,
            qbit_skip_checking: None,
        }
    }
}

impl OptionsContract for QbitOptions {
    type Partial = QbitOptionsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if self.inject_torrent {
            if self.qbit_url.is_none() {
                errors.push(NotSet("qBittorrent URL".to_owned()));
            }
            if self.qbit_username.is_none() {
                errors.push(NotSet("qBittorrent username".to_owned()));
            }
            if self.qbit_password.is_none() {
                errors.push(NotSet("qBittorrent password".to_owned()));
            }
        }
        if let Some(url) = &self.qbit_url
            && !url.starts_with("https://")
            && !url.starts_with("http://")
        {
            errors.push(UrlNotHttp("qBittorrent URL".to_owned(), url.clone()));
        }
    }
}
