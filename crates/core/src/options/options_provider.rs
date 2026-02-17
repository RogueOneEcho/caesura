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
    yaml: Option<String>,
    pub(crate) errors: Vec<OptionRule>,
}

impl OptionsProvider {
    /// Create a new [`OptionsProvider`] by reading CLI args and the config file.
    pub fn new() -> Self {
        let args = ArgumentsParser::get();
        let cli_options = ConfigOptionsPartial::from_args(&args);
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
        self.register(ConfigOptionsPartial::from_args(&args), services);
        self.register(CopyOptionsPartial::from_args(&args), services);
        self.register(FileOptionsPartial::from_args(&args), services);
        self.register(QueueAddArgsPartial::from_args(&args), services);
        self.register(RunnerOptionsPartial::from_args(&args), services);
        self.register(SoxOptionsPartial::from_args(&args), services);
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
/// - Returns `None` if the command does not use shared options
/// - Returns `None` if the file does not exist (validation reports the error)
/// - Falls back to the default config path if `--config` is not set
#[expect(clippy::ref_option, reason = "caller has Option<T>, not &T")]
fn read_config_file(options: &Option<ConfigOptionsPartial>) -> Option<String> {
    let options = options.as_ref()?;
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
