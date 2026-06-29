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

impl RunnerOptions {
    /// Effective CPU count, clamped to at least 1.
    ///
    /// - Guards against a `0` value stalling a bounded semaphore or `buffered` stream
    #[expect(clippy::as_conversions, reason = "u16 to usize is safe")]
    pub fn get_cpus(&self) -> usize {
        self.cpus.expect("cpus should be set").max(1) as usize
    }
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
    fn validate(&self, _validator: &mut OptionsValidator) {}
}
