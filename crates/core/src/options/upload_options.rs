use crate::prelude::*;
use caesura_macros::Options;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Supported torrent clients for direct API injection.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TorrentClient {
    Qbittorrent,
}

/// Options for upload
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Torrent client for direct API injection.
    #[arg(long)]
    pub torrent_client: Option<TorrentClient>,

    /// Torrent client API base URL.
    ///
    /// Example: `http://127.0.0.1:8080`
    #[arg(long)]
    pub torrent_client_url: Option<String>,

    /// Torrent client username.
    #[arg(long)]
    pub torrent_client_username: Option<String>,

    /// Torrent client password.
    #[arg(long)]
    pub torrent_client_password: Option<String>,

    /// Torrent client save path for injected torrents.
    ///
    /// This is currently used for qBittorrent.
    /// For remote clients, this must be a path visible to the client host.
    #[arg(long)]
    pub torrent_client_savepath: Option<String>,

    /// Torrent client category for injected torrents.
    ///
    /// This is currently used for qBittorrent.
    #[arg(long)]
    pub torrent_client_category: Option<String>,

    /// Torrent client tags for injected torrents.
    ///
    /// This is currently used for qBittorrent.
    #[arg(long)]
    pub torrent_client_tags: Option<Vec<String>>,

    /// Add injected torrents in paused state.
    #[arg(long)]
    pub torrent_client_paused: bool,

    /// Skip hash checking when injecting torrents.
    #[arg(long)]
    pub torrent_client_skip_checking: bool,

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
        let use_torrent_client = self.torrent_client.is_some()
            || self.torrent_client_url.is_some()
            || self.torrent_client_username.is_some()
            || self.torrent_client_password.is_some()
            || self.torrent_client_savepath.is_some()
            || self.torrent_client_category.is_some()
            || self
                .torrent_client_tags
                .as_ref()
                .is_some_and(|tags| !tags.is_empty())
            || self.torrent_client_paused
            || self.torrent_client_skip_checking;
        if use_torrent_client {
            if self.torrent_client.is_none() {
                errors.push(NotSet("Torrent client".to_owned()));
            }
            if self.torrent_client_url.is_none() {
                errors.push(NotSet("Torrent client URL".to_owned()));
            }
            if self.torrent_client_username.is_none() {
                errors.push(NotSet("Torrent client username".to_owned()));
            }
            if self.torrent_client_password.is_none() {
                errors.push(NotSet("Torrent client password".to_owned()));
            }
        }
        if let Some(url) = &self.torrent_client_url
            && !url.starts_with("https://")
            && !url.starts_with("http://")
        {
            errors.push(UrlNotHttp("Torrent client URL".to_owned(), url.clone()));
        }
    }
}
