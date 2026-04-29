use crate::prelude::*;
use std::cmp::Ordering;
use std::convert::Infallible;
use std::time::Duration;

/// URL of the RED indexer.
pub const RED_URL: &str = "https://redacted.sh";
/// URL of the previous RED indexer.
pub const RED_URL_CH: &str = "https://redacted.ch";
/// URL of the OPS indexer.
pub const OPS_URL: &str = "https://orpheus.network";
/// Tracker announce host for RED.
pub const RED_TRACKER_URL: &str = "https://flacsfor.me";
/// Tracker announce host for OPS.
pub const OPS_TRACKER_URL: &str = "https://home.opsfet.ch";

/// Indexer that a [`QueueItem`] belongs to.
///
/// - Deserializes case-insensitively from a string
/// - Serializes as a lowercase string
/// - Unknown values are normalized to lowercase and preserved in [`Indexer::Other`]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(from = "String", into = "String")]
pub enum Indexer {
    #[default]
    Red,
    Pth,
    Ops,
    /// Any indexer not listed above.
    ///
    /// The contained [`String`] must be lowercase. Constructing via [`From`] or
    /// [`FromStr`] enforces this, so direct construction should be avoided.
    Other(String),
}

impl Indexer {
    /// Lowercase string form of the indexer.
    #[must_use]
    pub fn as_lowercase(&self) -> &str {
        match self {
            Indexer::Red => "red",
            Indexer::Pth => "pth",
            Indexer::Ops => "ops",
            Indexer::Other(value) => value,
        }
    }

    /// Uppercase string form of the indexer.
    #[must_use]
    pub fn to_uppercase(&self) -> String {
        self.as_lowercase().to_uppercase()
    }

    /// Gazelle API rate limit for this indexer as `(requests, window)`.
    ///
    /// - RED allows 10 per 10s with API key auth; we use 8 to leave headroom
    /// - OPS applies per-action limits with `browse` capped at 5 per 10s; we use 4 to leave headroom
    /// - Other Gazelle-based trackers fall back to a conservative default
    #[must_use]
    pub fn gazelle_rate_limit(&self) -> (usize, Duration) {
        match self {
            Indexer::Red => (8, Duration::from_secs(10)),
            Indexer::Ops => (4, Duration::from_secs(10)),
            _ => (5, Duration::from_secs(10)),
        }
    }

    /// Delays between retry attempts when the API returns `TooManyRequests`.
    ///
    /// - OPS enforces a longer cooldown so uses `10s, 20s`
    /// - Other Gazelle-based indexers use `5s, 10s`
    /// - Up to 3 attempts total before the error propagates
    #[must_use]
    pub fn gazelle_retry_delays(&self) -> Vec<Duration> {
        match self {
            Indexer::Ops => vec![Duration::from_secs(10), Duration::from_secs(20)],
            _ => vec![Duration::from_secs(5), Duration::from_secs(10)],
        }
    }

    /// Check if `other` is the same indexer as this one, allowing known alternatives.
    ///
    /// - Asymmetric: if this is `Red`, `Pth` is accepted as an alternative
    /// - The reverse does not hold: `Pth` does not match `Red`
    pub fn match_with_alts(&self, other: &Indexer) -> bool {
        self == other || (self == &Indexer::Red && other == &Indexer::Pth)
    }
}

impl From<&str> for Indexer {
    fn from(value: &str) -> Self {
        let lowercase = value.to_lowercase();
        match lowercase.as_str() {
            "red" => Indexer::Red,
            "pth" => Indexer::Pth,
            "ops" => Indexer::Ops,
            _ => Indexer::Other(lowercase),
        }
    }
}

impl From<String> for Indexer {
    fn from(value: String) -> Self {
        Indexer::from(value.as_str())
    }
}

impl From<Indexer> for String {
    fn from(value: Indexer) -> Self {
        value.as_lowercase().to_owned()
    }
}

impl FromStr for Indexer {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Indexer::from(value))
    }
}

impl Ord for Indexer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_lowercase().cmp(other.as_lowercase())
    }
}

impl PartialOrd for Indexer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Indexer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.to_uppercase(), f)
    }
}
