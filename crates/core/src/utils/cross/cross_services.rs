use crate::prelude::*;

/// Services for interacting with the cross indexer during `cross`.
pub struct CrossServices {
    /// Torrent file provider configured for the cross indexer.
    pub(crate) torrent_files: Ref<TorrentFileProvider>,
    /// Checker that looks up matching torrents on the cross indexer.
    pub(crate) checker: Ref<CrossSeedChecker>,
}

impl CrossServices {
    /// Create a new [`CrossServices`] from the main and cross indexer options.
    ///
    /// - Returns `None` if the cross indexer config is missing or invalid.
    pub(crate) fn create(
        main_options: Ref<SharedOptions>,
        cache_options: Ref<CacheOptions>,
        file_options: Ref<FileOptions>,
        cross_config_options: Ref<CrossConfigOptions>,
    ) -> Option<Self> {
        let shared_options = Ref::new(cross_config_options.load_shared_options().ok()?);
        let paths = Ref::new(PathManager {
            shared_options: shared_options.clone(),
            cache_options,
            file_options,
        });
        let api = create_api(shared_options.clone());
        let torrent_files = Ref::new(TorrentFileProvider {
            paths,
            api: api.clone(),
        });
        let checker = Ref::new(CrossSeedChecker {
            api,
            main: main_options.get_indexer(),
            cross: shared_options.get_indexer(),
        });
        Some(Self {
            torrent_files,
            checker,
        })
    }
}

#[cfg(test)]
impl CrossServices {
    /// Create a [`CrossServices`] using a caller-supplied API client.
    ///
    /// - Mirrors [`CrossServices::create`] but skips the live API construction so tests
    ///   can inject a [`MockGazelleClient`].
    pub(crate) fn mock(
        main_options: Ref<SharedOptions>,
        cache_options: Ref<CacheOptions>,
        file_options: Ref<FileOptions>,
        cross_config_options: Ref<CrossConfigOptions>,
        api: Ref<Box<dyn GazelleClientTrait + Send + Sync>>,
    ) -> Option<Self> {
        let shared_options = Ref::new(cross_config_options.load_shared_options().ok()?);
        let paths = Ref::new(PathManager {
            shared_options: shared_options.clone(),
            cache_options,
            file_options,
        });
        let torrent_files = Ref::new(TorrentFileProvider {
            paths,
            api: api.clone(),
        });
        let checker = Ref::new(CrossSeedChecker {
            api,
            main: main_options.get_indexer(),
            cross: shared_options.get_indexer(),
        });
        Some(Self {
            torrent_files,
            checker,
        })
    }
}

fn create_api(shared: Ref<SharedOptions>) -> Ref<Box<dyn GazelleClientTrait + Send + Sync>> {
    let factory = GazelleClientFactory {
        options: GazelleClientOptions {
            url: shared.indexer_url.clone(),
            key: shared.api_key.clone(),
            user_agent: app_user_agent(true),
            requests_allowed_per_duration: None,
            request_limit_duration: None,
        },
    };
    Ref::new(Box::new(factory.create()))
}
