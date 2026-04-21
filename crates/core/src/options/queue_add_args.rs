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

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if let Some(path) = &self.queue_add_path {
            if !path.exists() {
                errors.push(OptionRule::DoesNotExist(
                    "Queue add path".to_owned(),
                    path.to_string_lossy().to_string(),
                ));
            }
        } else {
            errors.push(OptionRule::NotSet("Queue add path".to_owned()));
        }
    }
}
