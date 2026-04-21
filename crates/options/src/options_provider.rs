//! Setup helper for resolving, validating, and registering options with DI.

use crate::{
    ArgsProviderContract, Documented, OptionRule, OptionsPartialContract, OptionsRegistration,
};
use di::{ServiceCollection, existing_as_self};
use inventory::iter as inventory_iter;
use serde_yaml::from_str as from_yaml_str;
use std::sync::Arc;

/// Setup helper for resolving, validating, and registering options with DI.
///
/// Created during host setup to:
/// 1. Determine which options are needed for the current command
/// 2. Resolve options from CLI args and config file
/// 3. Validate all relevant options
/// 4. Collect errors for checking before host is built
/// 5. Register valid options with the DI container
#[derive(Default)]
pub struct OptionsProvider {
    args: Option<Arc<dyn ArgsProviderContract>>,
    yaml: Option<String>,
    /// Validation errors collected during registration.
    pub errors: Vec<OptionRule>,
}

impl OptionsProvider {
    /// Create an [`OptionsProvider`] from parsed CLI arguments.
    ///
    /// - `yaml`: config file contents (None if file missing)
    #[must_use]
    #[expect(
        clippy::as_conversions,
        reason = "required for Arc<dyn ArgsProviderContract> coercion"
    )]
    pub fn from_args<A: ArgsProviderContract + 'static>(
        args: Arc<A>,
        yaml: Option<String>,
    ) -> Self {
        Self {
            args: Some(args as Arc<dyn ArgsProviderContract>),
            yaml,
            errors: Vec::new(),
        }
    }

    fn merge_from_yaml<P>(&mut self, partial: &mut P)
    where
        P: OptionsPartialContract,
    {
        let Some(yaml) = &self.yaml else { return };
        match from_yaml_str(yaml) {
            Ok(file_partial) => {
                partial.merge(file_partial);
            }
            Err(error) => {
                self.errors
                    .push(OptionRule::ConfigDeserialize(error.to_string()));
            }
        }
    }

    /// Register a partial options type, extracting from [`clap::ArgMatches`] if applicable.
    pub fn register<P>(&mut self, services: &mut ServiceCollection)
    where
        P: OptionsPartialContract,
        P::Resolved: Documented + Send + Sync + 'static,
    {
        let name = P::Resolved::doc_metadata().name;
        let validate = self.args.as_ref().is_some_and(|a| a.uses_options(name));
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
    fn parse_cli_or_default<P>(&mut self, applicable: bool) -> P
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
        P::from_arg_matches(args.arg_matches()).unwrap_or_else(|error| {
            let name = String::from(P::Resolved::doc_metadata().name);
            self.errors
                .push(OptionRule::CliExtract(name, error.to_string()));
            P::default()
        })
    }

    /// Validate all relevant options and register them with DI.
    fn register_all(&mut self, services: &mut ServiceCollection) {
        for entry in inventory_iter::<OptionsRegistration> {
            (entry.register)(self, services);
        }
    }

    /// Create an [`OptionsProvider`] from YAML content without CLI arguments.
    #[must_use]
    pub fn from_yaml(yaml: Option<String>) -> Self {
        Self {
            args: None,
            yaml,
            errors: Vec::new(),
        }
    }

    /// Returns `true` if there are validation errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
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
