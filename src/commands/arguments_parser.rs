use clap::{CommandFactory, Parser};
use log::debug;
use std::process::exit;

use crate::commands::*;
/// Command line argument parser.
#[derive(Parser)]
#[command(version, about)]
pub struct ArgumentsParser {
    /// The command to run
    #[command(subcommand)]
    pub command: Option<CommandArguments>,
}

impl ArgumentsParser {
    /// Get the [`CommandArguments`] by parsing the arguments.
    ///
    /// Exiting triggers the clap help documentation etc to be displayed.
    #[must_use]
    pub(crate) fn get_or_show_help() -> CommandArguments {
        match ArgumentsParser::try_parse() {
            Ok(cli) => cli.command.unwrap_or_else(|| {
                debug!("No command provided. Showing help documentation:\n");
                ArgumentsParser::command()
                    .print_help()
                    .expect("Help should always print");
                exit(1);
            }),
            Err(error) => error.exit(),
        }
    }

    /// Get the [`CommandArguments`] by parsing the arguments.
    #[must_use]
    pub(crate) fn get() -> Option<CommandArguments> {
        match ArgumentsParser::try_parse().ok() {
            Some(cli) => cli.command,
            None => None,
        }
    }
}
