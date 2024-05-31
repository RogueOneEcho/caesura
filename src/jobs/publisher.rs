use crate::jobs::*;
use crate::logging::Verbosity;
use crate::options::SharedOptions;
use di::{injectable, Ref};

/// A publisher notifies subscribers when the status of a [Job] changes.
///
/// An [observer design pattern](https://refactoring.guru/design-patterns/observer) allows different
/// subscribers to be implemented independently.
pub struct Publisher {
    subscribers: Vec<Ref<dyn Subscriber + Send + Sync>>,
}

#[injectable]
impl Publisher {
    #[must_use]
    pub fn new(
        options: Ref<SharedOptions>,
        debug_subscriber: Ref<DebugSubscriber>,
        progress_bar_subscriber: Ref<ProgressBarSubscriber>,
    ) -> Self {
        let subscriber: Ref<dyn Subscriber + Send + Sync> =
            if options.verbosity.expect("Options should be set").as_num()
                >= Verbosity::Trace.as_num()
            {
                debug_subscriber
            } else {
                progress_bar_subscriber
            };
        Self {
            subscribers: vec![subscriber],
        }
    }
}

impl Subscriber for Publisher {
    /// Called when a new scope is started.
    fn start(&self, scope_id: &str) {
        for subscriber in &self.subscribers {
            subscriber.start(scope_id);
        }
    }

    /// Called when a scope is finished.
    fn finish(&self, scope_id: &str) {
        for subscriber in &self.subscribers {
            subscriber.finish(scope_id);
        }
    }

    /// Called when the status of a job changes.
    fn update(&self, command_id: &str, status: Status) {
        for subscriber in &self.subscribers {
            subscriber.update(command_id, status.clone());
        }
    }
}
