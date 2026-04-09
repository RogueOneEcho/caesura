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

    /// qBittorrent API base URL
    ///
    /// The base URL for your qBittorrent instance
    ///
    /// Examples: `http://localhost:8080`, `http://qbit`, `https://qbit.example.com`
    ///
    /// Or, the proxy URL with key if using [qui reverse proxy](https://getqui.com/docs/features/reverse-proxy)
    ///
    /// Examples:
    /// - `http://localhost:7476/proxy/YOUR_CLIENT_PROXY_KEY`
    /// - `https://qui.example.com/proxy/YOUR_CLIENT_PROXY_KEY`
    #[arg(long)]
    pub qbit_url: Option<String>,

    /// qBittorrent username.
    ///
    /// Not required when using qui reverse proxy.
    #[arg(long)]
    pub qbit_username: Option<String>,

    /// qBittorrent password.
    ///
    /// Not required when using qui reverse proxy.
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
    /// Build `AddTorrentOptions` for the qBittorrent API from these options.
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

    /// Push [`OptionRule`] violations for missing connection fields.
    pub(crate) fn validate_connection(&self, errors: &mut Vec<OptionRule>) {
        if self.qbit_url.is_none() {
            errors.push(NotSet("qBittorrent URL".to_owned()));
        }
        if self.requires_credentials() {
            if self.qbit_username.is_none() {
                errors.push(NotSet("qBittorrent username".to_owned()));
            }
            if self.qbit_password.is_none() {
                errors.push(NotSet("qBittorrent password".to_owned()));
            }
        }
    }

    /// Whether [`qbit_username`](Self::qbit_username) and
    /// [`qbit_password`](Self::qbit_password) must be set.
    ///
    /// - Returns `false` if using [qui reverse proxy](https://getqui.com/docs/features/reverse-proxy)
    /// - Returns `true` otherwise
    fn requires_credentials(&self) -> bool {
        !self
            .qbit_url
            .as_ref()
            .is_some_and(|url| url.contains("/proxy/"))
    }

    /// Create a [`QbitOptions`] with mock values for testing.
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
            self.validate_connection(errors);
        }
        if let Some(url) = &self.qbit_url
            && !url.starts_with("https://")
            && !url.starts_with("http://")
        {
            errors.push(UrlNotHttp("qBittorrent URL".to_owned(), url.clone()));
        }
        if let Some(url) = &self.qbit_url
            && url.ends_with('/')
        {
            errors.push(UrlInvalidSuffix("qBittorrent URL".to_owned(), url.clone()));
        }
    }
}
