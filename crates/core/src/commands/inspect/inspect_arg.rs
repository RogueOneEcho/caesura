use crate::commands::CommandArguments::*;
use crate::prelude::*;
use clap::Args;

/// Path argument for the inspect command.
#[derive(Args, Clone, Debug)]
pub struct InspectArg {
    /// Path to directory containing audio files.
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
}

#[injectable]
impl InspectArg {
    fn new() -> Self {
        Self::from_args().expect("inspect arg should be available when InspectCommand runs")
    }

    /// Get from command line arguments.
    #[must_use]
    pub fn from_args() -> Option<Self> {
        match ArgumentsParser::get() {
            Some(Inspect { arg, .. }) => Some(arg),
            _ => None,
        }
    }
}
