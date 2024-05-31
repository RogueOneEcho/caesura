use clap::Parser;

use crate::cli::sub_command::SubCommand;
use crate::options::SubCommand::{Spectrogram, Transcode, Verify};
use crate::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};

/// Options for all commands
#[derive(Parser)]
#[command(version, about)]
pub struct Arguments {
    /// The command to run
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

impl Arguments {
    /// Get the command from the command line arguments
    #[must_use]
    pub fn get_command_or_exit() -> SubCommand {
        match Arguments::try_parse() {
            Ok(cli) => cli.command.expect("Parsed arguments should have a command"),
            Err(error) => error.exit(),
        }
    }

    /// Get the command from the command line arguments
    #[must_use]
    pub fn get_command() -> Option<SubCommand> {
        match Arguments::try_parse().ok() {
            Some(cli) => cli.command,
            None => None,
        }
    }

    /// Get [`SharedOptions`] from the command line arguments
    #[must_use]
    pub fn get_shared_options() -> Option<SharedOptions> {
        match Arguments::get_command() {
            Some(Spectrogram { shared, .. }) => Some(shared),
            Some(Transcode { shared, .. }) => Some(shared),
            Some(Verify { shared, .. }) => Some(shared),
            _ => None,
        }
    }

    /// Get [`TranscodeOptions`] from the command line arguments
    #[must_use]
    pub fn get_transcode_options() -> Option<TranscodeOptions> {
        match Arguments::get_command() {
            Some(Transcode { transcode, .. }) => Some(transcode),
            Some(Verify { transcode, .. }) => Some(transcode),
            _ => None,
        }
    }

    /// Get [`SpectrogramOptions`] from the command line arguments
    #[must_use]
    pub fn get_spectrogram_options() -> Option<SpectrogramOptions> {
        match Arguments::get_command() {
            Some(Spectrogram { spectrogram, .. }) => Some(spectrogram),
            _ => None,
        }
    }
}
