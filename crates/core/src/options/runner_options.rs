use crate::prelude::*;
use num_cpus::get as get_num_cpus;

/// Options for concurrency
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct RunnerOptions {
    /// Number of cpus to use for processing.
    #[arg(long)]
    #[options(default_fn = default_cpus, default_doc = "Total CPUs")]
    pub cpus: Option<u16>,
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "default_fn signature requires Option for or_else chaining"
)]
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
fn default_cpus(_partial: &RunnerOptionsPartial) -> Option<u16> {
    Some(get_num_cpus() as u16)
}

impl OptionsContract for RunnerOptions {
    type Partial = RunnerOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}
