//! Check for newer releases on GitHub.

use crate::prelude::*;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use reqwest::Error as ReqwestError;

const RELEASES_API: &str = "https://api.github.com/repos/RogueOneEcho/caesura/releases/latest";
const RELEASES_URL: &str = "https://github.com/RogueOneEcho/caesura/releases";

/// Subset of the GitHub release API response.
///
/// <https://docs.github.com/en/rest/releases/releases#get-the-latest-release>
#[derive(Deserialize)]
pub(crate) struct GitHubRelease {
    /// Release tag name, e.g. `v0.27.2`.
    tag_name: String,
    /// When the release was published.
    published_at: DateTime<Utc>,
}

impl GitHubRelease {
    /// Check GitHub for a newer release and print a hint if one is available.
    pub(crate) async fn check_for_update() {
        let current = app_version_or_describe();
        let Ok(latest) = Self::get_latest().await else {
            return;
        };
        if latest.tag_name == current {
            trace!(
                "Version matches the latest GitHub release: {}",
                latest.tag_name
            );
        } else {
            let age = format_duration(Utc::now() - latest.published_at);
            warn!("{} is available", "Update".bold());
            info!(
                "Latest version is {}. Released {} ago",
                latest.tag_name, age
            );
            info!("{RELEASES_URL}");
        }
    }

    /// Fetch the latest release from the GitHub API.
    async fn get_latest() -> Result<GitHubRelease, ReqwestError> {
        Client::new()
            .get(RELEASES_API)
            .header("User-Agent", app_user_agent(true))
            .send()
            .await?
            .json()
            .await
    }
}

/// Format a human-friendly duration.
fn format_duration(delta: TimeDelta) -> String {
    let mins = delta.num_minutes() % 60;
    let hours = delta.num_hours() % 24;
    let days = delta.num_days() % 7;
    let weeks = delta.num_weeks();
    if weeks > 0 {
        format!("{weeks} weeks, {days} days")
    } else if days > 0 {
        format!("{days} days, {hours} hours")
    } else if hours > 0 {
        format!("{hours} hours, {mins} minutes")
    } else {
        format!("{mins} minutes")
    }
}
