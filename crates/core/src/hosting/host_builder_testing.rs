use crate::testing_prelude::*;
use di::{existing_as_self, singleton_as_self};

impl HostBuilder {
    /// Create a new [`HostBuilder`] for testing.
    ///
    /// - Registers mock Gazelle and qBittorrent clients so tests never build real HTTP clients
    /// - Tests MAY override the mocks with [`HostBuilder::with_mock_client`] or [`HostBuilder::with_mock_torrent_client`]
    #[must_use]
    pub(crate) fn mock() -> Self {
        let options = OptionsProvider::default();
        let mut builder = Self::new(options, None);
        let _ = builder
            .with_mock_client(MockGazelleClient::new())
            .with_mock_torrent_client(MockQBittorrentClient::default());
        builder
    }

    /// Register custom options for testing.
    #[must_use]
    pub fn with_options<T: Send + Sync + 'static>(&mut self, options: T) -> &mut Self {
        self.services.add(existing_as_self(options));
        self
    }

    /// Register a mock API client built from an [`AlbumConfig`].
    #[must_use]
    pub fn with_mock_api(&mut self, album_config: AlbumConfig) -> &mut Self {
        self.with_mock_client(album_config.api())
    }

    /// Register a pre-configured mock Gazelle API client for testing.
    #[must_use]
    #[expect(
        clippy::as_conversions,
        reason = "required for DI trait object registration"
    )]
    pub fn with_mock_client(&mut self, client: MockGazelleClient) -> &mut Self {
        let client: Ref<GazelleClient> = Ref::new(Box::new(client) as GazelleClient);
        self.services
            .add(singleton_as_self().from(move |_| client.clone()));
        self
    }

    /// Register a mock Gazelle API client for the cross indexer.
    ///
    /// - Replaces the default [`CrossServices`] factory so the cross indexer's
    ///   API client comes from the supplied mock instead of a live HTTP client.
    #[must_use]
    #[expect(
        clippy::as_conversions,
        reason = "required for DI trait object registration"
    )]
    pub fn with_mock_cross_client(&mut self, client: MockGazelleClient) -> &mut Self {
        let api: Ref<GazelleClient> = Ref::new(Box::new(client) as GazelleClient);
        self.services.add(singleton_as_self().from(move |services| {
            let main_options = services.get_required::<SharedOptions>();
            let cache_options = services.get_required::<CacheOptions>();
            let file_options = services.get_required::<FileOptions>();
            let cross_config_options = services.get_required::<CrossConfigOptions>();
            Ref::new(CrossServices::mock(
                main_options,
                cache_options,
                file_options,
                cross_config_options,
                api.clone(),
            ))
        }));
        self
    }

    /// Register a mock qBittorrent client for testing.
    #[must_use]
    #[expect(
        clippy::as_conversions,
        reason = "required for DI trait object registration"
    )]
    pub fn with_mock_torrent_client(&mut self, client: MockQBittorrentClient) -> &mut Self {
        let client: Ref<QbitClient> = Ref::new(Box::new(client) as QbitClient);
        self.services
            .add(singleton_as_self().from(move |_| client.clone()));
        self
    }

    /// Configure test options for the builder.
    ///
    /// - Sets up content, output, and cache directories
    pub async fn with_test_options(&mut self, test_dir: &TestDirectory) -> &mut Self {
        let output_dir = test_dir.output();
        let cache_dir = test_dir.cache();
        let reports_dir = test_dir.reports();
        tokio_create_dir_all(&output_dir)
            .await
            .expect("should be able to create output dir");
        tokio_create_dir_all(&cache_dir)
            .await
            .expect("should be able to create cache dir");
        self.with_options(SharedOptions {
            content: vec![SAMPLE_SOURCES_DIR.clone()],
            output: output_dir,
            ..SharedOptions::mock()
        })
        .with_options(CacheOptions { cache: cache_dir })
        .with_options(ReportOptions {
            reports_dir,
            no_reports: false,
        })
    }

    /// Build the [`Host`], panicking on error.
    ///
    /// Intended for tests where build errors indicate a test setup bug.
    #[must_use]
    #[expect(clippy::panic, reason = "intentional panic for test failures")]
    pub fn expect_build(&self) -> Host {
        match self.build() {
            Ok(host) => host,
            Err(error) => panic!("{error}"),
        }
    }
}
