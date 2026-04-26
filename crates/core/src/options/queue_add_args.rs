use crate::prelude::*;

/// Options for `queue add` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QueueAddArgs {
    /// A path to either a directory of `.torrent` files or a single YAML queue file.
    ///
    /// If you set this to the directory your torrent client stores `.torrent` files then caesura
    /// will automatically load everything from your client.
    /// - For qBittorrent use the `BT_backup` directory
    /// - For deluge use the `state` directory
    ///
    /// Examples:
    /// - `/srv/qBittorrent/BT_backup`
    /// - `/srv/deluge/state`
    /// - `./queue.yml`
    #[arg(value_name = "PATH")]
    pub queue_add_path: Option<PathBuf>,
}

impl OptionsContract for QueueAddArgs {
    type Partial = QueueAddArgsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_set("queue_add_path", &self.queue_add_path);
        if let Some(path) = &self.queue_add_path
            && !path.exists()
        {
            validator.push(OptionIssue::path_not_found("queue_add_path", path));
        }
    }
}
