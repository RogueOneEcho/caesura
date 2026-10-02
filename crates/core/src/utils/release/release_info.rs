//! Information about a GitHub release.

use crate::prelude::*;
use chrono::{DateTime, TimeDelta, Utc};

const RELEASES_URL: &str = "https://github.com/RogueOneEcho/caesura/releases";

/// Subset of the GitHub release API response.
///
/// <https://docs.github.com/en/rest/releases/releases#get-the-latest-release>
#[derive(Deserialize)]
pub(crate) struct ReleaseInfo {
    /// Release tag name, e.g. `v0.27.2`.
    tag_name: String,
    /// When the release was published.
    published_at: DateTime<Utc>,
}

impl ReleaseInfo {
    /// Log a hint if this release is newer than the running version.
    pub(crate) fn log(&self) {
        let current = app_version_or_describe();
        if self.tag_name == current {
            trace!(
                "Version matches the latest GitHub release: {}",
                self.tag_name
            );
        } else {
            let age = format_duration(Utc::now() - self.published_at);
            warn!("{} is available", "Update".bold());
            info!("Latest version is {}. Released {} ago", self.tag_name, age);
            info!("{RELEASES_URL}");
        }
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
