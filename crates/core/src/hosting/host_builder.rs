use crate::prelude::*;
use di::{Injectable, Mut, ServiceCollection, ServiceProvider, singleton_as_self};
use qbittorrent_api::{QBittorrentClientFactory, QBittorrentClientOptions};
use rogue_logging::InitLog;

/// Builder for configuring and constructing the application host.
pub struct HostBuilder {
    /// Service collection for dependency injection registration.
    pub services: ServiceCollection,
    /// Options provider for validation and registration.
    options: OptionsProvider,
}

impl HostBuilder {
    /// Create a new [`HostBuilder`] from CLI arguments and config file.
    #[must_use]
    pub fn new_cli() -> Self {
        let args = Arc::new(ArgumentsProvider::new());
        let yaml = read_config_file(&args);
        let options = OptionsProvider::from_args(args.clone(), yaml);
        Self::new(options, Some(args))
    }

    /// Wire up all services and register options with DI.
    #[must_use]
    pub(super) fn new(mut options: OptionsProvider, args: Option<Arc<ArgumentsProvider>>) -> Self {
        let mut services = ServiceCollection::new();
        services.register_options(&mut options);
        if let Some(args) = args {
            let args = args.clone();
            services.add(singleton_as_self().from(move |_| args.clone()));
        }
        services
            // Add main services
            .add(singleton_as_self().from(logger_factory))
            .add(Shutdown::singleton())
            .add(SoxFactory::singleton())
            .add(PathManager::transient())
            .add(IdProvider::transient())
            .add(SourceProvider::transient())
            .add(singleton_as_self().from(gazelle_factory))
            .add(singleton_as_self().from(qbit_factory))
            .add(JobRunner::transient())
            .add(Publisher::transient())
            .add(DebugSubscriber::transient())
            .add(ProgressBarSubscriber::transient())
            .add(ExistingFormatProvider::transient())
            .add(TargetFormatProvider::transient())
            // Add audit services
            .add(AuditCommand::transient())
            .add(TorrentAuditor::transient())
            // Add batch services
            .add(BatchCommand::transient())
            // Add config services
            .add(ConfigCommand::transient())
            // Add cross services
            .add(CrossCommand::transient())
            .add(singleton_as_self().from(cross_factory))
            // Add docs services
            .add(DocsCommand::transient())
            // Add inspect services
            .add(InspectCommand::transient())
            // Add queue services
            .add(QueueAddCommand::transient())
            .add(QueueFetchCommand::transient())
            .add(QueueListCommand::transient())
            .add(QueueRemoveCommand::transient())
            .add(QueueSummaryCommand::transient())
            .add(Queue::singleton())
            // Add spectrogram services
            .add(SpectrogramCommand::transient())
            .add(SpectrogramJobFactory::transient())
            .add(singleton_as_self().from(semaphore_factory))
            .add(singleton_as_self().from(joinset_factory))
            // Add transcode services
            .add(TranscodeCommand::transient())
            .add(TranscodeJobFactory::transient())
            .add(AdditionalJobFactory::transient())
            // Add shared injection services
            .add(TorrentInjector::transient())
            // Add upload services
            .add(UploadCommand::transient())
            // Add verify services
            .add(ApiVerifier::transient())
            .add(ContentVerifier::transient())
            .add(DecodeVerifier::transient())
            .add(FlacVerifier::transient())
            .add(TorrentFileProvider::transient())
            .add(VerifyCommand::transient())
            // Add report services
            .add(ReportRenderer::transient())
            .add(SourceReporter::transient())
            // Add version services
            .add(ReleaseProvider::transient())
            .add(VersionCommand::transient());
        HostBuilder { services, options }
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
fn qbit_factory(provider: &ServiceProvider) -> Arc<QbitClient> {
    let options = provider.get_required::<QbitOptions>();
    let client_options = match &options.qbit_url {
        Some(url) => QBittorrentClientOptions {
            host: url.clone(),
            username: options.qbit_username.clone().unwrap_or_default(),
            password: options.qbit_password.clone().unwrap_or_default(),
            user_agent: Some(app_user_agent(true)),
            ..QBittorrentClientOptions::default()
        },
        None => QBittorrentClientOptions::default(),
    };
    let factory = QBittorrentClientFactory {
        options: client_options,
    };
    Ref::new(Box::new(factory.create()) as QbitClient)
}

#[expect(clippy::as_conversions, reason = "required for traits")]
fn gazelle_factory(services: &ServiceProvider) -> Ref<GazelleClient> {
    let options = services.get_required::<SharedOptions>();
    let indexer = options.get_indexer();
    let (num, per) = indexer.gazelle_rate_limit();
    let factory = GazelleClientFactory {
        options: GazelleClientOptions {
            url: options.indexer_url.clone(),
            key: options.api_key.clone(),
            user_agent: app_user_agent(true),
            requests_allowed_per_duration: Some(num),
            request_limit_duration: Some(per),
            retry_delays: indexer.gazelle_retry_delays(),
        },
    };
    Ref::new(Box::new(factory.create()) as GazelleClient)
}

fn cross_factory(services: &ServiceProvider) -> Ref<Option<CrossServices>> {
    let main_options = services.get_required::<SharedOptions>();
    let cache_options = services.get_required::<CacheOptions>();
    let file_options = services.get_required::<FileOptions>();
    let cross_config_options = services.get_required::<CrossConfigOptions>();
    Ref::new(CrossServices::create(
        main_options,
        cache_options,
        file_options,
        cross_config_options,
    ))
}

fn logger_factory(provider: &ServiceProvider) -> Ref<Logger> {
    let options = provider.get_required::<SharedOptions>();
    let logger = Ref::new(
        default_logger()
            .with_verbosity(options.verbosity)
            .with_time_format(options.log_time)
            .create(),
    );
    logger.clone().init();
    logger
}

#[expect(clippy::type_complexity, reason = "collection of job results")]
fn joinset_factory(
    _services: &ServiceProvider,
) -> Ref<Mut<JoinSet<Result<(), Failure<JobAction>>>>> {
    let set: JoinSet<Result<(), Failure<JobAction>>> = JoinSet::new();
    RefMut::new(Mut::new(set))
}

fn semaphore_factory(services: &ServiceProvider) -> Ref<Semaphore> {
    let options = services.get_required::<RunnerOptions>();
    Ref::new(Semaphore::new(options.get_cpus()))
}
