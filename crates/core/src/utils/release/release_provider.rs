//! Fetch the latest release from GitHub.

use crate::prelude::*;
use reqwest::{Client, Response};

const RELEASES_API: &str = "https://api.github.com/repos/RogueOneEcho/caesura/releases/latest";

/// Fetch the latest caesura release from the GitHub API.
#[injectable]
pub(crate) struct ReleaseProvider {
    release_options: Ref<ReleaseOptions>,
}

impl ReleaseProvider {
    /// Fetch the latest release from the GitHub API.
    ///
    /// # Errors
    /// - [`ReleaseAction::Offline`] if `--offline` is set
    /// - [`ReleaseAction::BuildClient`] if the HTTP client can't be built, e.g. no CA certificates
    /// - [`ReleaseAction::Request`] if the request fails or returns an error status
    /// - [`ReleaseAction::Deserialize`] if the response isn't a valid release
    pub(crate) async fn get_latest(&self) -> Result<ReleaseInfo, Failure<ReleaseAction>> {
        if self.release_options.offline {
            return Err(Failure::from_action(ReleaseAction::Offline));
        }
        let client = Client::builder()
            .build()
            .map_err(Failure::wrap(ReleaseAction::BuildClient))?;
        let response = client
            .get(RELEASES_API)
            .header("User-Agent", app_user_agent(true))
            .send()
            .await
            .and_then(Response::error_for_status)
            .map_err(Failure::wrap_with(ReleaseAction::Request, |f| {
                f.with_url(RELEASES_API)
            }))?;
        response
            .json()
            .await
            .map_err(Failure::wrap(ReleaseAction::Deserialize))
    }
}

/// Errors returned by [`ReleaseProvider`]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub(crate) enum ReleaseAction {
    #[error("check for updates because `--offline` is set")]
    Offline,
    #[error("build HTTP client for update check")]
    BuildClient,
    #[error("request latest release")]
    Request,
    #[error("deserialize latest release")]
    Deserialize,
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    #[tokio::test]
    async fn release_provider_get_latest_offline() {
        // Arrange
        let host = HostBuilder::new()
            .with_options(ReleaseOptions { offline: true })
            .expect_build();
        let provider = host.services.get_required::<ReleaseProvider>();

        // Act
        let failure = provider
            .get_latest()
            .await
            .err()
            .expect("should fail when offline");

        // Assert
        assert_eq!(*failure.action(), ReleaseAction::Offline);
    }
}
