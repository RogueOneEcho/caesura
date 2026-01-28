use colored::Colorize;
use di::{Injectable, Mut, Ref, RefMut, ServiceCollection, singleton_as_self};
use log::error;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::commands::*;
use crate::hosting::*;
use crate::options::*;
use crate::utils::*;

use crate::built_info::{PKG_HOMEPAGE, PKG_NAME, PKG_VERSION};
use gazelle_api::{GazelleClientFactory, GazelleClientOptions, GazelleClientTrait};
use rogue_logging::{Error, LoggerBuilder};

/// Builder for configuring and constructing the application host.
pub struct HostBuilder {
    /// Service collection for dependency injection registration.
    pub services: ServiceCollection,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
    /// Create a new [`HostBuilder`] with default service registrations.
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub fn new() -> HostBuilder {
        let mut this = HostBuilder {
            services: ServiceCollection::new(),
        };
        this.services
            // Add options
            .add(OptionsProvider::singleton())
            .add(BatchOptions::singleton())
            .add(CacheOptions::singleton())
            .add(CopyOptions::singleton())
            .add(FileOptions::singleton())
            .add(RunnerOptions::singleton())
            .add(SharedOptions::singleton())
            .add(SourceArg::singleton())
            .add(SpectrogramOptions::singleton())
            .add(TargetOptions::singleton())
            .add(QueueAddArgs::singleton())
            .add(QueueRemoveArgs::singleton())
            .add(UploadOptions::singleton())
            .add(VerifyOptions::singleton())
            // Add main services
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                LoggerBuilder::new()
                    .with_exclude_filter("reqwest".to_owned())
                    .with_exclude_filter("cookie".to_owned())
                    .with_exclude_filter("lofty".to_owned())
                    .with_verbosity(options.verbosity.expect("verbosity should be set"))
                    .with_time_format(options.log_time.expect("log_time should be set"))
                    .create()
            }))
            .add(PathManager::transient())
            .add(IdProvider::transient())
            .add(SourceProvider::transient())
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                let factory = GazelleClientFactory {
                    options: GazelleClientOptions {
                        url: options
                            .indexer_url
                            .clone()
                            .expect("indexer_url should be set"),
                        key: options.api_key.clone().expect("api_key should be set"),
                        user_agent: format!("{PKG_NAME}/{PKG_VERSION} ({PKG_HOMEPAGE})"),
                        requests_allowed_per_duration: None,
                        request_limit_duration: None,
                    },
                };
                Ref::new(Box::new(factory.create()) as Box<dyn GazelleClientTrait + Send + Sync>)
            }))
            .add(JobRunner::transient())
            .add(Publisher::transient())
            .add(DebugSubscriber::transient())
            .add(ProgressBarSubscriber::transient())
            .add(TargetFormatProvider::transient())
            // Add config services
            .add(ConfigCommand::transient())
            // Add batch services
            .add(BatchCommand::transient())
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
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<RunnerOptions>();
                let cpus = options.cpus.expect("cpus should be set") as usize;
                Arc::new(Semaphore::new(cpus))
            }))
            .add(singleton_as_self().from(|_| {
                let set: JoinSet<Result<(), Error>> = JoinSet::new();
                RefMut::new(Mut::new(set))
            }))
            // Add transcode services
            .add(TranscodeCommand::transient())
            .add(TranscodeJobFactory::transient())
            .add(AdditionalJobFactory::transient())
            // Add upload services
            .add(UploadCommand::transient())
            // Add verify services
            .add(VerifyCommand::transient());
        this
    }

    /// Register custom options for testing.
    #[must_use]
    #[cfg(test)]
    pub fn with_options<T: Options + 'static>(&mut self, options: T) -> &mut Self {
        self.services
            .add(singleton_as_self().from(move |_| Ref::new(options.clone())));
        self
    }

    /// Register a mock API client for testing.
    #[must_use]
    #[cfg(test)]
    #[allow(clippy::as_conversions)]
    pub fn with_mock_api(&mut self, album_config: AlbumConfig) -> &mut Self {
        let client = album_config.api();
        let client: Ref<Box<dyn GazelleClientTrait + Send + Sync>> =
            Ref::new(Box::new(client) as Box<dyn GazelleClientTrait + Send + Sync>);
        self.services
            .add(singleton_as_self().from(move |_| client.clone()));
        self
    }

    /// Configure test options for the builder.
    ///
    /// - Sets up content, output, and cache directories
    /// - Configures target formats (FLAC, 320, V0)
    #[cfg(test)]
    pub async fn with_test_options(&mut self, test_dir: &TestDirectory) -> &mut Self {
        use crate::utils::TargetFormat::{_320, Flac, V0};
        use rogue_logging::{TimeFormat, Verbosity};
        use std::path::PathBuf;
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
            content: Some(vec![PathBuf::from(SAMPLE_SOURCES_DIR)]),
            output: Some(output_dir),
            verbosity: Some(Verbosity::Debug),
            log_time: Some(TimeFormat::None),
            indexer: Some("red".to_owned()),
            indexer_url: Some("https://redacted.sh".to_owned()),
            announce_url: Some("https://flacsfor.me/test/announce".to_owned()),
            api_key: Some("test_api_key".to_owned()),
            ..SharedOptions::default()
        })
        .with_options(TargetOptions {
            allow_existing: None,
            target: Some(vec![Flac, _320, V0]),
            sox_random_dither: Some(false),
        })
        .with_options(CacheOptions {
            cache: Some(cache_dir),
        })
        .with_options(BatchOptions {
            spectrogram: Some(false),
            transcode: Some(false),
            retry_transcode: Some(false),
            upload: Some(false),
            ..BatchOptions::default()
        })
    }

    /// Build the [`Host`] from the configured services.
    #[must_use]
    pub fn build(&self) -> Host {
        match self.services.build_provider() {
            Ok(services) => Host::new(services),
            Err(error) => {
                let _ = LoggerBuilder::new().create();
                error!("{} to build the application:", "Failed".bold());
                error!("{error}");
                exit(1)
            }
        }
    }
}
