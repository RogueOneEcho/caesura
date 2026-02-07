use crate::prelude::*;
use di::{ServiceCollection, existing_as_self};
use std::fs::read_to_string;

pub const DEFAULT_CONFIG_PATH: &str = "config.yml";

/// Setup helper for resolving, validating, and registering options with DI.
///
/// Created as a field on [`HostBuilder`] and used during setup to:
/// 1. Determine which options are needed for the current command
/// 2. Resolve options from CLI args and config file
/// 3. Validate all relevant options
/// 4. Collect errors for checking before host is built
/// 5. Register valid options with the DI container
pub struct OptionsProvider {
    yaml: Option<String>,
    pub(crate) errors: Vec<OptionRule>,
}

impl OptionsProvider {
    pub fn new() -> Self {
        let args = ArgumentsParser::get();
        let cli_options = SharedOptionsPartial::from_args(&args);
        Self {
            yaml: read_config_file(&cli_options),
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

    fn register<P>(&mut self, partial: Option<P>, services: &mut ServiceCollection)
    where
        P: OptionsPartialContract,
        P::Resolved: Send + Sync + 'static,
    {
        let is_applicable = partial.is_some();
        let mut partial = partial.unwrap_or_default();
        self.merge_from_yaml(&mut partial);
        let resolved = if is_applicable {
            match partial.resolve() {
                Ok(resolved) => resolved,
                Err(mut errors) => {
                    self.errors.append(&mut errors);
                    return;
                }
            }
        } else {
            partial.resolve_without_validation()
        };
        services.add(existing_as_self(resolved));
    }

    /// Validate all relevant options and register them with DI.
    fn register_all(&mut self, services: &mut ServiceCollection) {
        let args = ArgumentsParser::get();
        self.register(SharedOptionsPartial::from_args(&args), services);
        self.register(BatchOptionsPartial::from_args(&args), services);
        self.register(CacheOptionsPartial::from_args(&args), services);
        self.register(CopyOptionsPartial::from_args(&args), services);
        self.register(FileOptionsPartial::from_args(&args), services);
        self.register(RunnerOptionsPartial::from_args(&args), services);
        self.register(SpectrogramOptionsPartial::from_args(&args), services);
        self.register(TargetOptionsPartial::from_args(&args), services);
        self.register(UploadOptionsPartial::from_args(&args), services);
        self.register(VerifyOptionsPartial::from_args(&args), services);
    }

    /// Returns `true` if there are validation errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Read the config file.
///
/// Use the default config path if no path is set on the command line.
#[expect(clippy::ref_option, reason = "caller has Option<T>, not &T")]
fn read_config_file(options: &Option<SharedOptionsPartial>) -> Option<String> {
    let path = options
        .as_ref()
        .and_then(|o| o.config.clone())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    match read_to_string(path) {
        Ok(yaml) => Some(yaml),
        Err(e) => {
            init_logger();
            warn!("{} to read config file: {}", "Failed".bold(), e);
            None
        }
    }
}

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
