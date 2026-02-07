//! Shared logger configuration.

use rogue_logging::Verbosity::Trace;
use rogue_logging::{InitLog, LoggerBuilder};

/// Create a [`LoggerBuilder`] with standard exclude filters applied.
#[must_use]
pub fn default_logger() -> LoggerBuilder {
    LoggerBuilder::new()
        .with_exclude_filter("reqwest".to_owned())
        .with_exclude_filter("cookie".to_owned())
        .with_exclude_filter("lofty".to_owned())
        .with_verbosity(Trace)
}

/// Initialize a logger with standard filters and [`Trace`] verbosity.
pub fn init_logger() {
    default_logger().create().init();
}
