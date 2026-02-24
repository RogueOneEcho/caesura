//! Setup helper for resolving, validating, and registering options with DI.

use crate::prelude::*;
use di::{ServiceCollection, existing_as_self};
use std::fs::read_to_string;

/// Setup helper for resolving, validating, and registering options with DI.
///
/// Created as a field on [`HostBuilder`] and used during setup to:
/// 1. Determine which options are needed for the current command
/// 2. Resolve options from CLI args and config file
/// 3. Validate all relevant options
/// 4. Collect errors for checking before host is built
/// 5. Register valid options with the DI container
pub struct OptionsProvider {
    args: Option<Ref<ArgumentsProvider>>,
    yaml: Option<String>,
    pub(crate) errors: Vec<OptionRule>,
}

impl OptionsProvider {
    /// Create a new [`OptionsProvider`] by reading CLI args and the config file.
    pub fn new(args: Option<Ref<ArgumentsProvider>>) -> Self {
        let yaml = read_config_file(&args);
        Self {
            args,
            yaml,
            errors: Vec::new(),
        }
    }

    fn merge_from_yaml<P>(&self, partial: &mut P)
    where
        P: OptionsPartialContract,
    {
        let Some(yaml) = &self.yaml else { return };
        match serde_yaml::from_str(yaml) {
            Ok(file_partial) => {
                partial.merge(file_partial);
            }
            Err(error) => {
                init_logger();
                error!("{} to deserialize config file: {}", "Failed".bold(), error);
            }
        }
    }

    /// Register a partial options type, extracting from [`ArgMatches`] if applicable.
    fn register<P>(&mut self, services: &mut ServiceCollection)
    where
        P: OptionsPartialContract,
        P::Resolved: Documented + Send + Sync + 'static,
    {
        let name = P::Resolved::doc_metadata().name;
        let validate = self
            .args
            .as_ref()
            .is_some_and(|a| a.get_command().uses_options(name));
        let mut partial = self.parse_cli_or_default::<P>(validate);
        self.merge_from_yaml(&mut partial);
        if validate {
            match partial.resolve() {
                Ok(resolved) => {
                    services.add(existing_as_self(resolved));
                }
                Err(mut errors) => self.errors.append(&mut errors),
            }
        } else {
            services.add(existing_as_self(partial.resolve_without_validation()));
        }
    }

    /// Parse CLI arguments into a partial, or return defaults if not applicable.
    fn parse_cli_or_default<P>(&self, applicable: bool) -> P
    where
        P: OptionsPartialContract,
        P::Resolved: Documented,
    {
        if !applicable {
            return P::default();
        }
        let Some(args) = self.args.as_ref() else {
            return P::default();
        };
        args.get_args::<P>().unwrap_or_else(|error| {
            error!(
                "{} to extract {} from CLI arguments: {}",
                "Failed".bold(),
                P::Resolved::doc_metadata().name,
                error,
            );
            P::default()
        })
    }

    /// Validate all relevant options and register them with DI.
    fn register_all(&mut self, services: &mut ServiceCollection) {
        self.register::<BatchOptionsPartial>(services);
        self.register::<CacheOptionsPartial>(services);
        self.register::<ConfigOptionsPartial>(services);
        self.register::<CopyOptionsPartial>(services);
        self.register::<FileOptionsPartial>(services);
        self.register::<InspectArgPartial>(services);
        self.register::<QueueAddArgsPartial>(services);
        self.register::<QueueRemoveArgsPartial>(services);
        self.register::<RunnerOptionsPartial>(services);
        self.register::<SharedOptionsPartial>(services);
        self.register::<SourceArgPartial>(services);
        self.register::<SoxOptionsPartial>(services);
        self.register::<SpectrogramOptionsPartial>(services);
        self.register::<TargetOptionsPartial>(services);
        self.register::<UploadOptionsPartial>(services);
        self.register::<VerifyOptionsPartial>(services);
    }

    /// Returns `true` if there are validation errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Read the config file.
///
/// - Returns `None` if the command does not use config options
/// - Returns `None` if the file does not exist (validation reports the error)
/// - Falls back to the default config path if `--config` is not set
#[expect(clippy::ref_option, reason = "shared reference avoids cloning the Ref")]
fn read_config_file(args: &Option<Ref<ArgumentsProvider>>) -> Option<String> {
    let options = args.as_deref()?.get_args::<ConfigOptionsPartial>().ok()?;
    let path = options
        .config
        .clone()
        .unwrap_or_else(PathManager::default_config_path);
    read_to_string(path).ok()
}

/// Extension trait for registering resolved options with a [`ServiceCollection`].
pub trait RegisterOptions {
    /// Register all options with the service collection.
    ///
    /// Returns the [`OptionsProvider`] for error checking.
    fn register_options(&mut self, provider: &mut OptionsProvider) -> &mut Self;
}

impl RegisterOptions for ServiceCollection {
    fn register_options(&mut self, provider: &mut OptionsProvider) -> &mut Self {
        provider.register_all(self);
        self
    }
}
