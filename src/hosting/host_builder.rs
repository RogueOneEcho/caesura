use colored::Colorize;
use di::{singleton_as_self, Injectable, Mut, Ref, RefMut, ServiceCollection};
use log::error;
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::batch::BatchCommand;
use crate::built_info::{PKG_HOMEPAGE, PKG_NAME, PKG_VERSION};
use crate::formats::TargetFormatProvider;
use crate::fs::PathManager;
use crate::hosting::Host;
use crate::jobs::{DebugSubscriber, JobRunner, ProgressBarSubscriber, Publisher};
use crate::options::config_command::ConfigCommand;
use crate::options::*;
use crate::queue::queue_summary_command::QueueSummaryCommand;
use crate::queue::{Queue, QueueAddCommand, QueueListCommand};
use crate::source::{IdProvider, SourceProvider};
use crate::spectrogram::{SpectrogramCommand, SpectrogramJobFactory};
use crate::transcode::{AdditionalJobFactory, TranscodeCommand, TranscodeJobFactory};
use crate::upload::UploadCommand;
use crate::verify::VerifyCommand;
use gazelle_api::GazelleClientFactory;
use rogue_logging::Error;
use rogue_logging::Logger;

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
            .add(UploadOptions::singleton())
            .add(VerifyOptions::singleton())
            // Add main services
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                let logger = Logger {
                    enabled_threshold: options.verbosity.expect("verbosity should be set"),
                    time_format: options.log_time.expect("verbosity should be set"),
                    start: SystemTime::now(),
                    package_name: PKG_NAME.to_owned(),
                };
                Ref::new(logger)
            }))
            .add(PathManager::transient())
            .add(IdProvider::transient())
            .add(SourceProvider::transient().as_mut())
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                let factory = GazelleClientFactory {
                    url: options
                        .indexer_url
                        .clone()
                        .expect("indexer_url should be set"),
                    key: options.api_key.clone().expect("api_key should be set"),
                    user_agent: format!("{PKG_NAME}/{PKG_VERSION} ({PKG_HOMEPAGE})"),
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
                Logger::force_init(PKG_NAME.to_owned());
                error!("{} to build the application:", "Failed".bold());
                error!("{error}");
                exit(1)
            }
        }
    }
}
