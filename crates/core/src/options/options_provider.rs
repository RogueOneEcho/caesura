use crate::prelude::*;
use di::{ServiceCollection, existing_as_self};
use rogue_logging::Verbosity::Trace;
use rogue_logging::{Logger, LoggerBuilder};
use std::fs::read_to_string;
use std::sync::Arc;

pub const DEFAULT_CONFIG_PATH: &str = "config.yml";

/// Setup helper for resolving, validating, and registering options with DI.
///
/// This is NOT a DI service. It's used during setup to:
/// 1. Determine which options are needed for the current command
/// 2. Resolve options from CLI args and config file
/// 3. Validate all relevant options
/// 4. Exit with clear errors if validation fails
/// 5. Register resolved options with the DI container
pub struct OptionsProvider {
    yaml: Option<String>,
    command: Option<Command>,
    pub(crate) errors: Vec<OptionRule>,
}

impl OptionsProvider {
    fn new() -> Self {
        let cli_options = SharedOptions::partial_from_args().unwrap_or_default();
        Self {
            yaml: read_config_file(&cli_options),
            command: Command::from_args(),
            errors: Vec::new(),
        }
    }

    fn merge_from_yaml<P>(&self, partial: &mut P)
    where
        P: OptionsPartial,
    {
        let Some(yaml) = &self.yaml else { return };
        match P::from_yaml(yaml) {
            Ok(file_partial) => {
                partial.merge(&file_partial);
            }
            Err(error) => {
                let _ = init_logger();
                error!("{} to deserialize config file: {}", "Failed".bold(), error);
            }
        }
    }

    fn register<P>(&mut self, services: &mut ServiceCollection)
    where
        P: OptionsPartial,
        P::Resolved: ApplicableCommands + Send + Sync + 'static,
    {
        let mut partial = P::from_args().unwrap_or_default();
        self.merge_from_yaml(&mut partial);
        if let Some(command) = &self.command
            && P::Resolved::applicable_commands().contains(command)
        {
            partial.validate(&mut self.errors);
        }
        let resolved = partial.resolve();
        services.add(existing_as_self(resolved));
    }

    /// Validate all relevant options and register them with DI.
    fn register_all(&mut self, services: &mut ServiceCollection) {
        self.register::<SharedOptionsPartial>(services);
        self.register::<BatchOptionsPartial>(services);
        self.register::<CacheOptionsPartial>(services);
        self.register::<CopyOptionsPartial>(services);
        self.register::<FileOptionsPartial>(services);
        self.register::<RunnerOptionsPartial>(services);
        self.register::<SpectrogramOptionsPartial>(services);
        self.register::<TargetOptionsPartial>(services);
        self.register::<UploadOptionsPartial>(services);
        self.register::<VerifyOptionsPartial>(services);
    }
}

/// Read the config file.
///
/// Use the default config path if no path is set on the command line.
fn read_config_file(options: &SharedOptionsPartial) -> Option<String> {
    let path = options
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    match read_to_string(path) {
        Ok(yaml) => Some(yaml),
        Err(e) => {
            let _ = init_logger();
            warn!("{} to read config file: {}", "Failed".bold(), e);
            None
        }
    }
}

fn init_logger() -> Arc<Logger> {
    LoggerBuilder::new()
        .with_exclude_filter("reqwest".to_owned())
        .with_exclude_filter("cookie".to_owned())
        .with_verbosity(Trace)
        .create()
}

pub trait RegisterOptions {
    fn register_options(&mut self) -> &mut Self;
}

impl RegisterOptions for ServiceCollection {
    /// Register all relevant options for the current command with the DI container.
    ///
    /// This method:
    /// 1. Reads CLI args and config file
    /// 2. Determines the current command
    /// 3. Resolves and validates all options relevant to that command
    /// 4. Exits with clear error messages if validation fails
    /// 5. Registers the resolved options with DI
    fn register_options(&mut self) -> &mut Self {
        let mut provider = OptionsProvider::new();
        provider.register_all(self);
        self.add(existing_as_self(provider));
        self
    }
}
