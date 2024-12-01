use clap::Subcommand;

use crate::options::source_arg::SourceArg;
use crate::options::verify_options::VerifyOptions;
use crate::options::{
    BatchOptions, CacheOptions, CopyOptions, FileOptions, QueueAddArgs, RunnerOptions,
    SharedOptions, SpectrogramOptions, TargetOptions, UploadOptions,
};

/// Cli sub-commands and arguments
#[derive(Subcommand, Debug, Clone)]
pub enum CommandArguments {
    /// Read the config file if it exists and concatenate default values.
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
        copy: CopyOptions,
        #[command(flatten)]
        file: FileOptions,
        #[command(flatten)]
        batch: BatchOptions,
        #[command(flatten)]
        cache: CacheOptions,
    },

    /// Add FLAC sources to the queue without transcoding
    Queue {
        #[command(subcommand)]
        command: QueueCommandArguments,
    },

    /// Generate spectrograms for each track of a FLAC source.
    Spectrogram {
        #[command(flatten)]
        source: SourceArg,
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
        source: SourceArg,
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        copy: CopyOptions,
        #[command(flatten)]
        file: FileOptions,
        #[command(flatten)]
        runner: RunnerOptions,
    },

    /// Upload transcodes of a FLAC source.
    Upload {
        #[command(flatten)]
        source: SourceArg,
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
        source: SourceArg,
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        target: TargetOptions,
        #[command(flatten)]
        verify: VerifyOptions,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum QueueCommandArguments {
    /// Add a directory of `.torrent` files to the queue
    Add {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        cache: CacheOptions,
        #[command(flatten)]
        args: QueueAddArgs,
    },

    /// List the sources in the queue
    List {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        cache: CacheOptions,
        #[command(flatten)]
        batch: BatchOptions,
    },

    /// Summarize the sources in the queue
    Summary {
        #[command(flatten)]
        shared: SharedOptions,
        #[command(flatten)]
        cache: CacheOptions,
    },
}
