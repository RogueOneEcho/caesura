use crate::prelude::*;

/// Options for report generation
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct ReportOptions {
    /// Path to the directory where generated reports are written.
    #[arg(long)]
    #[options(default_fn = default_reports_dir, default_doc = "`~/.local/share/caesura/output/reports/` or platform equivalent")]
    pub reports_dir: PathBuf,

    /// Disable automatic report generation.
    #[arg(long)]
    pub no_reports: bool,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "Options macro default_fn requires Option<T>"
)]
fn default_reports_dir(_partial: &ReportOptionsPartial) -> Option<PathBuf> {
    Some(PathManager::default_reports_dir())
}

impl OptionsContract for ReportOptions {
    type Partial = ReportOptionsPartial;

    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
