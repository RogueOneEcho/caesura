use serde::{Deserialize, Serialize};

use crate::options::OptionRule;
use caesura_macros::Options;

/// Options for concurrency
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Spectrogram, Transcode))]
#[options(field_name = "runner")]
#[options(defaults_fn = "Self::apply_calculated_defaults")]
pub struct RunnerOptions {
    /// Number of cpus to use for processing.
    ///
    /// Default: Total number of CPUs
    #[arg(long)]
    pub cpus: Option<u16>,
}

impl RunnerOptions {
    /// Apply calculated defaults that depend on runtime values.
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    pub fn apply_calculated_defaults(partial: &mut RunnerOptionsPartial) {
        if partial.cpus.is_none() {
            partial.cpus = Some(num_cpus::get() as u16);
        }
    }

    /// Validate the partial options.
    pub fn validate_partial(_: &RunnerOptionsPartial, _: &mut Vec<OptionRule>) {}
}

impl Default for RunnerOptions {
    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
    fn default() -> Self {
        Self {
            cpus: Some(num_cpus::get() as u16),
        }
    }
}
