use colored::Colorize;
#[cfg(test)]
use di::Ref;
use di::{singleton_as_self, Injectable, Mut, RefMut, ServiceCollection};
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
use gazelle_api::{GazelleClientFactory, GazelleClientOptions};
use rogue_logging::{Error, LoggerBuilder};

pub struct HostBuilder {
    pub services: ServiceCollection,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
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
            .add(SourceProvider::transient().as_mut())
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
                let api = factory.create();
                RefMut::new(Mut::new(api))
            }))
            .add(JobRunner::transient())
            .add(Publisher::transient())
            .add(DebugSubscriber::transient())
            .add(ProgressBarSubscriber::transient())
            .add(TargetFormatProvider::transient())
            // Add config services
            .add(ConfigCommand::transient())
            // Add batch services
            .add(BatchCommand::transient().as_mut())
            // Add queue services
            .add(QueueAddCommand::transient().as_mut())
            .add(QueueListCommand::transient().as_mut())
            .add(QueueRemoveCommand::transient().as_mut())
            .add(QueueSummaryCommand::transient().as_mut())
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<CacheOptions>();
                let queue = Queue::from_options(options);
                RefMut::new(Mut::new(queue))
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
            .add(UploadCommand::transient().as_mut())
            // Add verify services
            .add(VerifyCommand::transient().as_mut());
        this
    }

    #[must_use]
    #[cfg(test)]
    pub fn with_options<T: Options + 'static>(&mut self, options: T) -> &mut Self {
        self.services
            .add(singleton_as_self().from(move |_| Ref::new(options.clone())));
        self
    }

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
