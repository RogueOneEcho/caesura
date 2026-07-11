use crate::prelude::*;

/// Options for the `audit` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct AuditArgs {
    /// A single .torrent file or a directory of `.torrent` files to scan for problematic paths.
    ///
    /// Examples:
    /// - `/srv/qBittorrent/BT_backup`
    /// - `/srv/deluge/state`
    /// - `path/to/file.torrent`
    #[arg(value_name = "PATH")]
    pub audit_path: Option<PathBuf>,
}

impl OptionsContract for AuditArgs {
    type Partial = AuditArgsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_set("audit_path", &self.audit_path);
        if let Some(path) = &self.audit_path
            && !path.exists()
        {
            validator.push(OptionIssue::path_not_found("audit_path", path));
        }
    }
}
