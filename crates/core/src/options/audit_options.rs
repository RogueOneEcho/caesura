use crate::prelude::*;

/// Options controlling which problems the `audit` command reports.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "one ignore flag per check")]
pub struct AuditOptions {
    /// Should the check for non-UTF-8 paths be disabled?
    #[arg(long)]
    pub ignore_non_utf8: bool,
    /// Should the check for file torrents be disabled?
    #[arg(long)]
    pub ignore_single_file: bool,
    /// Should the check for libtorrent-stripped path characters be disabled?
    #[arg(long)]
    pub ignore_libtorrent: bool,
    /// Should the check for invisible or zero-width path characters be disabled?
    #[arg(long)]
    pub ignore_invisible: bool,
    /// Should the check for unsafe path segments be disabled?
    #[arg(long)]
    pub ignore_unsafe: bool,
    /// Should the check for decomposed (non-NFC) path characters be disabled?
    #[arg(long)]
    pub ignore_nfd: bool,
    /// Should the check for file extensions lost on disk be disabled?
    #[arg(long)]
    pub ignore_lost_extension: bool,
    /// Should diffs be rendered with BB code?
    #[arg(long)]
    pub print_bb_code: bool,
}

impl OptionsContract for AuditOptions {
    type Partial = AuditOptionsPartial;

    fn validate(&self, _validator: &mut OptionsValidator) {}
}
