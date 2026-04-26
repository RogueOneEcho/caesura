use crate::prelude::*;

/// Options for `queue fetch` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QueueFetchOptions {
    /// qBittorrent categories to discover torrents from.
    ///
    /// `queue fetch` queries the qBittorrent API filtered by these categories
    /// and adds any fully downloaded torrents that are not already in the queue.
    ///
    /// An empty string (`""`) fetches torrents that have no category assigned.
    #[arg(long)]
    #[options(required)]
    pub qbit_fetch_categories: Vec<String>,
}

impl OptionsContract for QueueFetchOptions {
    type Partial = QueueFetchOptionsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_non_empty("qbit_fetch_categories", &self.qbit_fetch_categories);
    }
}
