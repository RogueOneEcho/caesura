use crate::jobs::*;

/// A generic trait for subscribing to the status of a [Job].
///
/// The [`CommandRunner`] will call the [update] method when the status of
/// a [Job] changes.
///
/// An [observer design pattern](https://refactoring.guru/design-patterns/observer) allows different
/// subscribers to be implemented independently.
pub trait Subscriber {
    /// Called when a new scope is started.
    fn start(&self, scope_id: &str);

    /// Called when a scope is finished.
    fn finish(&self, scope_id: &str);

    /// Called when the status of a job changes.
    fn update(&self, job_id: &str, status: Status);
}
