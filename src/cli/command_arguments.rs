use clap::Subcommand;

use crate::options::verify_options::VerifyOptions;
use crate::options::{
    BatchOptions, FileOptions, RunnerOptions, SharedOptions, SpectrogramOptions, TargetOptions,
    UploadOptions,
};

/// Cli sub-commands and arguments
#[derive(Subcommand, Debug, Clone)]
pub enum CommandArguments {
    /// Generate a config.json file in the current working directory.
    Config,
    /// Verify, transcode, and upload from multiple FLAC sources in one command.
    Batch {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        verify: VerifyOptions,
        #[command(flatten)]
        runner: RunnerOptions,
        #[command(flatten)]
        spectrogram: SpectrogramOptions,
        #[command(flatten)]
        file: FileOptions,
        #[command(flatten)]
        batch: BatchOptions,
    },

    /// Generate spectrograms for each track of a FLAC source.
    Spectrogram {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        spectrogram: SpectrogramOptions,
        #[command(flatten)]
        runner: RunnerOptions,
    },

    /// Transcode each track of a FLAC source to the target formats.
    Transcode {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        file: FileOptions,
        #[command(flatten)]
        runner: RunnerOptions,
    },

    /// Upload transcodes of a FLAC source.
    Upload {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        upload: UploadOptions,
    },

    /// Verify a FLAC source is suitable for transcoding.
    Verify {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        verify: VerifyOptions,
    },
}
