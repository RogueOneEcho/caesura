//! Command line argument parser.

use crate::prelude::*;
use clap::{ArgMatches, CommandFactory, Error};
use std::process::exit;

/// Command line argument provider.
pub struct ArgumentsProvider {
    command: Command,
    matches: ArgMatches,
}

impl ArgumentsProvider {
    /// Create a new [`ArgumentsProvider`] by parsing CLI arguments.
    ///
    /// Exits if arguments are invalid or there is no sub command.
    #[must_use]
    pub fn new() -> Self {
        let matches = match Cli::command().try_get_matches() {
            Ok(matches) => matches,
            Err(error) => error.exit(),
        };
        let Some((command, matches)) = Command::resolve(&matches) else {
            trace!("No command provided. Showing help documentation:\n");
            Cli::command()
                .print_help()
                .expect("Help should always print");
            exit(1);
        };
        Self { command, matches }
    }

    /// Resolved command variant.
    #[must_use]
    pub fn get_command(&self) -> Command {
        self.command
    }

    /// Extract typed arguments from the resolved [`ArgMatches`].
    pub fn get_args<T: clap::FromArgMatches>(&self) -> Result<T, Error> {
        T::from_arg_matches(&self.matches)
    }
}
