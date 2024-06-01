use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter};
use serde::{Deserialize, Serialize};

use crate::logging::*;

/// Log level
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Silent,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl Verbosity {
    /// Get a number representing the verbosity.
    #[must_use]
    pub fn as_num(self) -> usize {
        match self {
            Silent => 0,
            Error => 1,
            Warn => 2,
            Info => 3,
            Debug => 4,
            Trace => 5,
        }
    }

    /// Get the colorized, uppercase, four letter id.
    #[must_use]
    pub fn get_id(self) -> ColoredString {
        match self {
            Silent => "NONE".dark_gray(),
            Error => "ERRO".red().dimmed(),
            Warn => "WARN".yellow().dimmed(),
            Info => "INFO".blue().dimmed(),
            Debug => "DBUG".dark_gray(),
            Trace => "TRCE".dark_gray(),
        }
    }

    /// Get the colorized, single character icon.
    #[must_use]
    pub fn get_icon(self) -> ColoredString {
        match self {
            Silent => " ".dark_gray(),
            Error => "!".red(),
            Warn => "⚠".yellow(),
            Info => "i".blue(),
            Debug => " ".dark_gray(),
            Trace => "↶".dark_gray(),
        }
    }

    #[must_use]
    pub fn from_level(level: Level) -> Self {
        match level {
            Level::Error => Error,
            Level::Warn => Warn,
            Level::Info => Info,
            Level::Debug => Debug,
            Level::Trace => Trace,
        }
    }

    #[must_use]
    pub fn to_level(self) -> Option<Level> {
        match self {
            Silent => None,
            Error => Some(Level::Error),
            Warn => Some(Level::Warn),
            Info => Some(Level::Info),
            Debug => Some(Level::Debug),
            Trace => Some(Level::Trace),
        }
    }

    #[must_use]
    pub fn to_level_filter(self) -> LevelFilter {
        match self {
            Silent => LevelFilter::Off,
            Error => LevelFilter::Error,
            Warn => LevelFilter::Warn,
            Info => LevelFilter::Info,
            Debug => LevelFilter::Debug,
            Trace => LevelFilter::Trace,
        }
    }
}
