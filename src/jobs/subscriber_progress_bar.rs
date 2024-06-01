use crate::errors::AppError;
use crate::jobs::*;
use crate::logging::*;
use colored::Colorize;
use di::{injectable, Ref, RefMut};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use tokio::task::JoinSet;

/// A [Subscriber] that updates a progress bar in the console
pub struct ProgressBarSubscriber {
    /// The set of jobs to track
    set: RefMut<JoinSet<Result<(), AppError>>>,

    /// The progress bar
    bar: ProgressBar,
}

#[injectable]
impl ProgressBarSubscriber {
    /// Create a new [`ProgressBarSubscriber`]
    pub fn new(logger: Ref<Logger>, set: RefMut<JoinSet<Result<(), AppError>>>) -> Self {
        let bar = create_progress_bar(logger);
        Self { set, bar }
    }
}

impl Subscriber for ProgressBarSubscriber {
    /// Called when a new scope is started.
    fn start(&self, _scope_id: &str) {
        let total = self
            .set
            .read()
            .expect("Should be able to read the job set")
            .len() as u64;
        self.bar.set_length(total);
    }

    /// Called when a scope is finished.
    fn finish(&self, _scope_id: &str) {
        self.bar.finish();
    }

    /// Called when the status of a job changes.
    fn update(&self, _job_id: &str, status: Status) {
        if matches!(status, Completed) {
            self.bar.inc(1);
        }
    }
}

fn create_progress_bar(logger: Ref<Logger>) -> ProgressBar {
    let bar = ProgressBar::new(100);
    let prefix = logger.format_prefix(Info);
    let template = format!(
        "{} [{}] {}{}/{{len}}  {} remain",
        prefix,
        "{bar:40}".blue(),
        "{elapsed:>3}",
        "{pos:>4}".gray(),
        "{eta}".gray()
    )
    .dark_gray();
    let style = ProgressStyle::default_bar()
        .template(template.to_string().as_str())
        .expect("Progress style should compile")
        .progress_chars("#>-");
    bar.set_style(style);
    bar.set_draw_target(ProgressDrawTarget::stderr());
    bar
}
