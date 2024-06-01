use di::{singleton_as_self, Ref};

use crate::hosting::{Host, HostBuilder};
use crate::options::{SharedOptions, SpectrogramOptions, TranscodeOptions};

pub struct TestHostBuilder {
    pub builder: HostBuilder,
}

impl TestHostBuilder {
    #[must_use]
    pub fn new() -> Self {
        let builder = HostBuilder::new();
        Self { builder }
    }

    #[must_use]
    pub fn with_shared(&mut self, options: SharedOptions) -> &mut TestHostBuilder {
        self.builder
            .services
            .add(singleton_as_self().from(move |_| Ref::new(options.clone())));
        self
    }

    #[must_use]
    pub fn with_spectrogram(&mut self, options: SpectrogramOptions) -> &mut TestHostBuilder {
        self.builder
            .services
            .add(singleton_as_self().from(move |_| Ref::new(options.clone())));
        self
    }

    #[must_use]
    pub fn with_transcode(&mut self, options: TranscodeOptions) -> &mut TestHostBuilder {
        self.builder
            .services
            .add(singleton_as_self().from(move |_| Ref::new(options.clone())));
        self
    }

    #[must_use]
    pub fn build(&self) -> Host {
        self.builder.build()
    }
}
