use rogue_logging::Verbosity::Trace;
use rogue_logging::{Logger, LoggerBuilder};
use std::sync::Arc;

pub fn init_logger() -> Arc<Logger> {
    LoggerBuilder::new()
        .with_exclude_filter("reqwest".to_owned())
        .with_exclude_filter("cookie".to_owned())
        .with_exclude_filter("lofty".to_owned())
        .with_verbosity(Trace)
        .create()
}
