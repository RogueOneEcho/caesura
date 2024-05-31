use clap::Subcommand;

use crate::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};

/// Cli sub-commands and arguments
#[derive(Subcommand, Debug, Clone)]
pub enum SubCommand {
    /// Spectrogram sub-command
    Spectrogram {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        spectrogram: SpectrogramOptions,
    },

    /// Spectrogram sub-command
    Transcode {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        transcode: TranscodeOptions,
    },

    /// Verify sub-command
    Verify {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        transcode: TranscodeOptions,
    },
}
