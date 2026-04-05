use crate::hosting::*;
use crate::prelude::*;
#[cfg(test)]
use di::existing_as_self;
use di::{Injectable, Mut, ServiceCollection, ServiceProvider, singleton_as_self};
use gazelle_api::{GazelleClientFactory, GazelleClientOptions, GazelleClientTrait};
use rogue_logging::InitLog;
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Builder for configuring and constructing the application host.
pub struct HostBuilder {
    /// Service collection for dependency injection registration.
    pub services: ServiceCollection,
    /// Options provider for validation and registration.
    options: OptionsProvider,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
    /// Create a new [`HostBuilder`] without CLI arguments.
    #[must_use]
    pub(crate) fn new() -> Self {
        let options = OptionsProvider::default();
        Self::new_internal(options, None)
    }

    /// Create a new [`HostBuilder`] from CLI arguments and config file.
    #[must_use]
    pub fn new_cli() -> Self {
        let args = Arc::new(ArgumentsProvider::new());
        let yaml = read_config_file(&args);
        let options = OptionsProvider::from_args(args.clone(), yaml);
        Self::new_internal(options, Some(args))
    }

    /// Wire up all services and register options with DI.
    #[must_use]
    fn new_internal(mut options: OptionsProvider, args: Option<Arc<ArgumentsProvider>>) -> Self {
        let mut services = ServiceCollection::new();
        services.register_options(&mut options);
        if let Some(args) = args {
            let args = args.clone();
            services.add(singleton_as_self().from(move |_| args.clone()));
        }
        services
            // Add main services
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                let logger = Ref::new(
                    default_logger()
                        .with_verbosity(options.verbosity)
                        .with_time_format(options.log_time)
                        .create(),
                );
                logger.clone().init();
                logger
            }))
            .add(SoxFactory::singleton())
            .add(PathManager::transient())
            .add(IdProvider::transient())
            .add(SourceProvider::transient())
            .add(singleton_as_self().from(gazelle_factory))
            .add(JobRunner::transient())
            .add(Publisher::transient())
            .add(DebugSubscriber::transient())
            .add(ProgressBarSubscriber::transient())
            .add(TargetFormatProvider::transient())
            // Add batch services
            .add(BatchCommand::transient())
            // Add config services
            .add(ConfigCommand::transient())
            // Add docs services
            .add(DocsCommand::transient())
            // Add inspect services
            .add(InspectCommand::transient())
            // Add queue services
            .add(QueueAddCommand::transient())
            .add(QueueListCommand::transient())
            .add(QueueRemoveCommand::transient())
            .add(QueueSummaryCommand::transient())
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<CacheOptions>();
                Ref::new(Queue::from_options(options))
            }))
            // Add spectrogram services
            .add(SpectrogramCommand::transient())
            .add(SpectrogramJobFactory::transient())
            .add(singleton_as_self().from(semaphore_factory))
            .add(singleton_as_self().from(joinset_factory))
            // Add transcode services
            .add(TranscodeCommand::transient())
            .add(TranscodeJobFactory::transient())
            .add(AdditionalJobFactory::transient())
            // Add upload services
            .add(UploadCommand::transient())
            // Add verify services
            .add(VerifyCommand::transient())
            // Add version services
            .add(VersionCommand::transient());
        HostBuilder { services, options }
    }

    /// Register custom options for testing.
    #[must_use]
    #[cfg(test)]
    pub fn with_options<T: Send + Sync + 'static>(&mut self, options: T) -> &mut Self {
        self.services.add(existing_as_self(options));
        self
    }

    /// Register a mock API client built from an [`AlbumConfig`].
    #[must_use]
    #[cfg(test)]
    pub fn with_mock_api(&mut self, album_config: AlbumConfig) -> &mut Self {
        self.with_mock_client(album_config.api())
    }

    /// Register a pre-configured mock API client for testing.
    #[must_use]
    #[cfg(test)]
    #[expect(
        clippy::as_conversions,
        reason = "required for DI trait object registration"
    )]
    pub fn with_mock_client(&mut self, client: gazelle_api::MockGazelleClient) -> &mut Self {
        let client: Ref<Box<dyn GazelleClientTrait + Send + Sync>> =
            Ref::new(Box::new(client) as Box<dyn GazelleClientTrait + Send + Sync>);
        self.services
            .add(singleton_as_self().from(move |_| client.clone()));
        self
    }

    /// Configure test options for the builder.
    ///
    /// - Sets up content, output, and cache directories
    #[cfg(test)]
    pub async fn with_test_options(&mut self, test_dir: &TestDirectory) -> &mut Self {
        use tokio::fs::create_dir_all;
        let output_dir = test_dir.output();
        let cache_dir = test_dir.cache();
        create_dir_all(&output_dir)
            .await
            .expect("should be able to create output dir");
        create_dir_all(&cache_dir)
            .await
            .expect("should be able to create cache dir");
        self.with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: output_dir,
            ..SharedOptions::mock()
        })
        .with_options(CacheOptions { cache: cache_dir })
    }

    /// Build the [`Host`] from the configured services.
    ///
    /// Returns an error if options validation or DI container building fails.
    pub fn build(&self) -> Result<Host, BuildError> {
        if self.options.has_errors() {
            return Err(BuildError::Options(self.options.errors.clone()));
        }
        let services = self.services.build_provider()?;
        Ok(Host::new(services))
    }

    /// Build the [`Host`], panicking on error.
    ///
    /// Intended for tests where build errors indicate a test setup bug.
    #[cfg(test)]
    #[must_use]
    #[expect(clippy::panic, reason = "intentional panic for test failures")]
    pub fn expect_build(&self) -> Host {
        match self.build() {
            Ok(host) => host,
            Err(error) => panic!("{error}"),
        }
    }
}

/// Read the config file.
///
/// - Returns `None` if the command does not use config options
/// - Returns `None` if the file does not exist (validation reports the error)
/// - Falls back to the default config path if `--config` is not set
fn read_config_file(args: &ArgumentsProvider) -> Option<String> {
    if !args.get_command().uses_options("ConfigOptions") {
        return None;
    }
    let options = args.get_args::<ConfigOptionsPartial>().ok()?;
    let path = options
        .config
        .clone()
        .unwrap_or_else(PathManager::default_config_path);
    read_to_string(path.expand_tilde()).ok()
}

#[expect(clippy::as_conversions, reason = "required for traits")]
fn gazelle_factory(services: &ServiceProvider) -> Ref<Box<dyn GazelleClientTrait + Send + Sync>> {
    let options = services.get_required::<SharedOptions>();
    let factory = GazelleClientFactory {
        options: GazelleClientOptions {
            url: options.indexer_url.clone(),
            key: options.api_key.clone(),
            user_agent: app_user_agent(true),
            requests_allowed_per_duration: None,
            request_limit_duration: None,
        },
    };
    Ref::new(Box::new(factory.create()) as Box<dyn GazelleClientTrait + Send + Sync>)
}

#[expect(clippy::type_complexity, reason = "collection of job results")]
fn joinset_factory(
    _services: &ServiceProvider,
) -> Ref<Mut<JoinSet<Result<(), Failure<JobAction>>>>> {
    let set: JoinSet<Result<(), Failure<JobAction>>> = JoinSet::new();
    RefMut::new(Mut::new(set))
}

#[expect(clippy::as_conversions, reason = "u16 to usize is safe")]
fn semaphore_factory(services: &ServiceProvider) -> Ref<Semaphore> {
    let options = services.get_required::<RunnerOptions>();
    let cpus = options.cpus.expect("cpus should be set") as usize;
    Ref::new(Semaphore::new(cpus))
}
