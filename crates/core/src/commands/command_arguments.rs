use clap::Subcommand;

use crate::options::*;

/// Cli sub-commands and arguments
#[derive(Subcommand, Debug, Clone)]
pub enum CommandArguments {
    /// Read the config file if it exists and concatenate default values.
    Config,

    /// Generate markdown documentation for configuration options.
    Docs,

    /// Verify, transcode, and upload from multiple FLAC sources in one command.
    Batch {
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        target: TargetOptionsPartial,
        #[command(flatten)]
        verify: VerifyOptionsPartial,
        #[command(flatten)]
        runner: RunnerOptionsPartial,
        #[command(flatten)]
        spectrogram: SpectrogramOptionsPartial,
        #[command(flatten)]
        copy: CopyOptionsPartial,
        #[command(flatten)]
        file: FileOptionsPartial,
        #[command(flatten)]
        batch: BatchOptionsPartial,
        #[command(flatten)]
        cache: CacheOptionsPartial,
        #[command(flatten)]
        upload: UploadOptionsPartial,
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
        shared: SharedOptionsPartial,
        #[command(flatten)]
        spectrogram: SpectrogramOptionsPartial,
        #[command(flatten)]
        runner: RunnerOptionsPartial,
    },

    /// Transcode each track of a FLAC source to the target formats.
    Transcode {
        #[command(flatten)]
        source: SourceArg,
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        target: TargetOptionsPartial,
        #[command(flatten)]
        copy: CopyOptionsPartial,
        #[command(flatten)]
        file: FileOptionsPartial,
        #[command(flatten)]
        runner: RunnerOptionsPartial,
    },

    /// Upload transcodes of a FLAC source.
    Upload {
        #[command(flatten)]
        source: SourceArg,
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        target: TargetOptionsPartial,
        #[command(flatten)]
        upload: UploadOptionsPartial,
        #[command(flatten)]
        copy: CopyOptionsPartial,
    },

    /// Verify a FLAC source is suitable for transcoding.
    Verify {
        #[command(flatten)]
        source: SourceArg,
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        target: TargetOptionsPartial,
        #[command(flatten)]
        verify: VerifyOptionsPartial,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum QueueCommandArguments {
    /// Add a directory of `.torrent` files to the queue
    Add {
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        cache: CacheOptionsPartial,
        #[command(flatten)]
        args: QueueAddArgs,
    },

    /// List the sources in the queue
    List {
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        cache: CacheOptionsPartial,
        #[command(flatten)]
        batch: BatchOptionsPartial,
    },

    /// Remove an item from the queue
    #[command(name = "rm")]
    Remove {
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        cache: CacheOptionsPartial,
        #[command(flatten)]
        args: QueueRemoveArgs,
    },

    /// Summarize the sources in the queue
    Summary {
        #[command(flatten)]
        shared: SharedOptionsPartial,
        #[command(flatten)]
        cache: CacheOptionsPartial,
    },
}
