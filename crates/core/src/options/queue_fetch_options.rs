use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Options for `queue fetch` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QueueFetchOptions {
    /// qBittorrent categories to discover torrents from.
    ///
    /// `queue fetch` queries the qBittorrent API filtered by these categories
    /// and adds any fully downloaded torrents that are not already in the queue.
    #[arg(long)]
    #[options(required)]
    pub qbit_fetch_categories: Vec<String>,
}

impl OptionsContract for QueueFetchOptions {
    type Partial = QueueFetchOptionsPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
