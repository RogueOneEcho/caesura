use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::commands::{ArgumentsParser, CommandArguments, QueueCommandArguments};

/// Identifies which CLI command is being executed.
///
/// This is used for command-specific validation during options resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    Batch,
    Config,
    Docs,
    Queue,
    Spectrogram,
    Transcode,
    Upload,
    Verify,
}

impl Command {
    /// Get the current command from the CLI arguments.
    ///
    /// Returns `None` if no command was specified or arguments could not be parsed.
    #[must_use]
    pub fn from_args() -> Option<Self> {
        ArgumentsParser::get().map(|cmd| Self::from(&cmd))
    }
}

impl From<&CommandArguments> for Command {
    fn from(args: &CommandArguments) -> Self {
        match args {
            CommandArguments::Batch { .. } => Command::Batch,
            CommandArguments::Config => Command::Config,
            CommandArguments::Docs => Command::Docs,
            CommandArguments::Queue { .. } => Command::Queue,
            CommandArguments::Spectrogram { .. } => Command::Spectrogram,
            CommandArguments::Transcode { .. } => Command::Transcode,
            CommandArguments::Upload { .. } => Command::Upload,
            CommandArguments::Verify { .. } => Command::Verify,
        }
    }
}

impl From<&QueueCommandArguments> for Command {
    fn from(_args: &QueueCommandArguments) -> Self {
        Command::Queue
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Command::Batch => write!(f, "batch"),
            Command::Config => write!(f, "config"),
            Command::Docs => write!(f, "docs"),
            Command::Queue => write!(f, "queue"),
            Command::Spectrogram => write!(f, "spectrogram"),
            Command::Transcode => write!(f, "transcode"),
            Command::Upload => write!(f, "upload"),
            Command::Verify => write!(f, "verify"),
        }
    }
}
