use crate::prelude::FmtResult;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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

/// Indexer that a [`QueueItem`](crate::commands::QueueItem) belongs to.
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
        self.to_uppercase().fmt(f)
    }
}
