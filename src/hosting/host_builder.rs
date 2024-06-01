use std::process::exit;
use std::sync::Arc;

use colored::Colorize;
use di::{singleton_as_self, Injectable, Mut, RefMut, ServiceCollection};
use log::error;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api::{Api, ApiFactory};
use crate::errors::AppError;
use crate::formats::TargetFormatProvider;
use crate::hosting::Host;
use crate::jobs::{DebugSubscriber, JobRunner, ProgressBarSubscriber, Publisher};
use crate::logging::{Logger, Trace};
use crate::options::{OptionsProvider, SharedOptions, SpectrogramOptions, TranscodeOptions};
use crate::source::SourceProvider;
use crate::spectrogram::{SpectrogramGenerator, SpectrogramJobFactory};
use crate::transcode::{AdditionalJobFactory, SourceTranscoder, TranscodeJobFactory};
use crate::verify::SourceVerifier;

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
    pub fn new() -> HostBuilder {
        let mut this = HostBuilder {
            services: ServiceCollection::new(),
        };
        this.services
            // Add options
            .add(OptionsProvider::singleton())
            .add(SharedOptions::singleton())
            .add(SpectrogramOptions::singleton())
            .add(TranscodeOptions::singleton())
            // Add main services
            .add(Logger::singleton())
            .add(SourceProvider::transient().as_mut())
            .add(ApiFactory::transient())
            .add(Api::singleton().as_mut())
            .add(JobRunner::transient())
            .add(Publisher::transient())
            .add(DebugSubscriber::transient())
            .add(ProgressBarSubscriber::transient())
            .add(TargetFormatProvider::transient())
            // Add transcode services
            .add(SourceTranscoder::transient())
            .add(TranscodeJobFactory::transient())
            .add(AdditionalJobFactory::transient())
            // Add spectrogram services
            .add(SpectrogramGenerator::transient())
            .add(SpectrogramJobFactory::transient())
            .add(singleton_as_self().from(|provider| {
                let options = provider.get_required::<SharedOptions>();
                let cpu_limit = options.cpu_limit.expect("Options should be set") as usize;
                Arc::new(Semaphore::new(cpu_limit))
            }))
            .add(singleton_as_self().from(|_| {
                let set: JoinSet<Result<(), AppError>> = JoinSet::new();
                RefMut::new(Mut::new(set))
            }))
            // Add verify services
            .add(SourceVerifier::transient().as_mut());
        this
    }

    #[must_use]
    pub fn build(&self) -> Host {
        match self.services.build_provider() {
            Ok(services) => Host::new(services),
            Err(error) => {
                Logger::init_new(Trace);
                error!("{} to build the application:", "Failed".red().bold());
                error!("{error}");
                exit(1)
            }
        }
    }
}
