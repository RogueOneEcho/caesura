//! Generic command line argument parser.

use crate::CommandEnumContract;
use clap::{ArgMatches, CommandFactory, Error, FromArgMatches};
use std::marker::PhantomData;
use std::process::exit;

/// Contract for accessing parsed CLI arguments without knowing the generic parameters.
///
/// Abstracts over `ArgumentsProvider<P, C>` so that [`OptionsProvider`](crate::OptionsProvider)
/// can remain non-generic (required for `inventory` registration).
pub trait ArgsProviderContract: Send + Sync {
    /// Check if the resolved command uses the given options type.
    fn uses_options(&self, name: &str) -> bool;
    /// Access the resolved [`ArgMatches`].
    fn arg_matches(&self) -> &ArgMatches;
}

/// Generic command line argument provider.
///
/// `P` is the clap parser struct (e.g., `Cli`).
/// `C` is the command enum (e.g., `Command`).
pub struct ArgsProvider<P, C> {
    /// Resolved command variant.
    command: C,
    /// Resolved argument matches for the active subcommand.
    matches: ArgMatches,
    _parser: PhantomData<P>,
}

impl<P: CommandFactory, C: CommandEnumContract> ArgsProvider<P, C> {
    /// Create a new [`ArgsProvider`] by parsing CLI arguments.
    ///
    /// Exits if arguments are invalid or there is no sub command.
    #[must_use]
    pub fn new() -> Self {
        let matches = match P::command().try_get_matches() {
            Ok(matches) => matches,
            Err(error) => error.exit(),
        };
        let Some((command, matches)) = C::resolve(&matches) else {
            P::command().print_help().expect("Help should always print");
            exit(1);
        };
        Self {
            command,
            matches,
            _parser: PhantomData,
        }
    }

    /// Resolved command variant.
    #[must_use]
    pub fn get_command(&self) -> C {
        self.command
    }

    /// Extract typed arguments from the resolved [`ArgMatches`].
    pub fn get_args<T: FromArgMatches>(&self) -> Result<T, Error> {
        T::from_arg_matches(&self.matches)
    }
}

impl<P: Send + Sync, C: CommandEnumContract + Send + Sync> ArgsProviderContract
    for ArgsProvider<P, C>
{
    fn uses_options(&self, name: &str) -> bool {
        self.command.uses_options(name)
    }
    fn arg_matches(&self) -> &ArgMatches {
        &self.matches
    }
}
