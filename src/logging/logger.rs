use std::time::SystemTime;

use colored::control::SHOULD_COLORIZE;
use colored::{ColoredString, Colorize};
use di::{injectable, Ref};
use log::*;

use crate::built_info;
use crate::logging::*;
use crate::options::SharedOptions;

pub struct Logger {
    pub enabled_threshold: Verbosity,
    pub show_timestamp: bool,
    pub start: SystemTime,
}

#[injectable]
impl Logger {
    #[must_use]
    pub fn new(options: Ref<SharedOptions>) -> Self {
        Self::from_verbosity(options.verbosity.unwrap_or_default())
    }

    fn from_verbosity(verbosity: Verbosity) -> Self {
        let show_timestamp = verbosity.as_num() >= Debug.as_num();
        let start = SystemTime::now();
        Self {
            enabled_threshold: verbosity,
            show_timestamp,
            start,
        }
    }

    pub fn init(logger: Ref<Logger>) {
        SHOULD_COLORIZE.set_override(true);
        let filter = logger.enabled_threshold.to_level_filter();
        let logger = Box::new(logger);
        if let Err(error) = set_boxed_logger(logger).map(|()| set_max_level(filter)) {
            trace!("{} to initialize the logger: {}", "Failed".bold(), error);
        }
    }

    pub fn init_new(verbosity: Verbosity) {
        Self::init(Ref::new(Logger::from_verbosity(verbosity)));
    }

    fn format_log(&self, verbosity: Verbosity, message: String) -> String {
        let prefix = self.format_prefix(verbosity);
        let message = self.format_message(verbosity, message);
        format!("{prefix} {message}")
    }

    fn format_message(&self, verbosity: Verbosity, message: String) -> String {
        if verbosity.as_num() >= Debug.as_num() {
            format!("{}", message.dimmed())
        } else {
            message
        }
    }

    #[must_use]
    pub fn format_prefix(&self, verbosity: Verbosity) -> String {
        let verbosity_id = verbosity.get_id();
        let icon = verbosity.get_icon();
        if self.show_timestamp {
            let time = self.format_time();
            format!("{time:>8} {verbosity_id} {icon}")
        } else {
            format!("{verbosity_id} {icon}")
        }
    }

    fn format_time(&self) -> ColoredString {
        let duration = self.start.elapsed().unwrap_or_default().as_secs_f64();
        format!("{duration:.3}").dark_gray()
    }

    fn is_enabled(&self, verbosity: Verbosity) -> bool {
        verbosity.as_num() <= self.enabled_threshold.as_num()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if !metadata.target().starts_with(built_info::PKG_NAME) {
            return false;
        }
        self.is_enabled(Verbosity::from_level(metadata.level()))
    }

    #[allow(clippy::print_stderr)]
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let verbosity = Verbosity::from_level(record.level());
            let message = format!("{}", record.args());
            let log = self.format_log(verbosity, message);
            eprintln!("{log}");
        }
    }

    fn flush(&self) {}
}
