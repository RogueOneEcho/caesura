//! Command enum as the single source of truth for CLI structure.

use crate::prelude::*;
use caesura_macros::CommandEnum;

/// Concrete [`ArgsProvider`] wired to the caesura CLI parser and command enum.
pub type ArgumentsProvider = ArgsProvider<Cli, Command>;

/// An all-in-one command line tool to transcode FLAC audio files
/// and upload to gazelle based indexers/trackers
#[derive(CommandEnum, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Command {
    /// Verify, transcode, and upload from multiple FLAC sources in one command.
    #[options(
        ConfigOptions,
        SharedOptions,
        VerifyOptions,
        TargetOptions,
        SpectrogramOptions,
        SoxOptions,
        CopyOptions,
        FileOptions,
        RunnerOptions,
        UploadOptions,
        QbitOptions,
        QbitUploadOptions,
        CacheOptions,
        BatchOptions
    )]
    Batch,

    /// Read the config file if it exists and concatenate default values.
    #[options(ConfigOptions)]
    Config,

    /// Find a source on the primary indexer and look up cross-seeds on the cross indexer.
    #[options(
        SourceArg,
        ConfigOptions,
        SharedOptions,
        CacheOptions,
        FileOptions,
        TargetOptions,
        CrossConfigOptions,
        CrossOptions,
        QbitOptions,
        QbitCrossOptions,
        CopyOptions
    )]
    Cross,

    /// Generate markdown documentation for configuration options.
    Docs,

    /// Inspect audio file metadata in a directory.
    #[options(InspectArg)]
    Inspect,

    /// Add FLAC sources to the queue without transcoding
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Queue(QueueCommand),

    /// Generate spectrograms for each track of a FLAC source.
    #[options(
        SourceArg,
        ConfigOptions,
        SharedOptions,
        SpectrogramOptions,
        SoxOptions,
        RunnerOptions
    )]
    Spectrogram,

    /// Transcode each track of a FLAC source to the target formats.
    #[options(
        SourceArg,
        ConfigOptions,
        SharedOptions,
        TargetOptions,
        SoxOptions,
        CopyOptions,
        FileOptions,
        RunnerOptions
    )]
    Transcode,

    /// Upload transcodes of a FLAC source.
    ///
    /// `FileOptions` is not listed despite `TranscodeJobFactory` depending on
    /// `PathManager` which injects it. Upload only uses `TranscodeJobFactory`
    /// to generate command strings for the description, where the output paths
    /// are overridden with placeholders (`output.{ext}`), so `rename_tracks`
    /// has no effect.
    #[options(
        SourceArg,
        ConfigOptions,
        SharedOptions,
        TargetOptions,
        UploadOptions,
        QbitOptions,
        QbitUploadOptions,
        CopyOptions
    )]
    Upload,

    /// Verify a FLAC source is suitable for transcoding.
    #[options(
        SourceArg,
        ConfigOptions,
        SharedOptions,
        CacheOptions,
        TargetOptions,
        VerifyOptions
    )]
    Verify,

    /// Display version information for caesura and dependencies.
    #[command(short_flag = 'V', long_flag = "version")]
    #[options(SoxOptions)]
    Version,
}

/// Sub-commands for the `queue` command group.
#[derive(CommandEnum, Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[command_enum(parent = "queue")]
pub enum QueueCommand {
    /// Add a directory of `.torrent` files to the queue
    #[options(ConfigOptions, SharedOptions, CacheOptions, QueueAddArgs)]
    Add,

    /// Discover torrents via the qBittorrent API and add them to the queue.
    #[options(
        ConfigOptions,
        SharedOptions,
        CacheOptions,
        QbitOptions,
        QueueFetchOptions
    )]
    Fetch,

    /// List the sources in the queue
    #[options(ConfigOptions, SharedOptions, CacheOptions, BatchOptions)]
    List,

    /// Remove an item from the queue
    #[cli_name = "rm"]
    #[options(QueueRemoveArgs, ConfigOptions, SharedOptions, CacheOptions)]
    Remove,

    /// Summarize the sources in the queue
    #[options(ConfigOptions, SharedOptions, CacheOptions)]
    Summary,
}
