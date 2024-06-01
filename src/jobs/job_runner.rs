use std::sync::Arc;

use crate::errors::AppError;
use di::{injectable, Ref, RefMut};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::jobs::*;

/// Execute a [Job] in parallel across a restricted number of threads.
///
/// [Semaphore] is used to limit the number of commands that can be executed concurrently.
/// [`JoinSet`] is used to execute commands in parallel, and collate the results.
/// [Publisher] is updated by an
/// [observer design pattern](https://refactoring.guru/design-patterns/observer) when the status
/// of a [Job] changes.
pub struct JobRunner {
    pub semaphore: Arc<Semaphore>,
    pub set: RefMut<JoinSet<Result<(), AppError>>>,
    pub publisher: Ref<Publisher>,
}

#[injectable]
impl JobRunner {
    /// Create a new [`JobRunner`].
    pub fn new(
        semaphore: Arc<Semaphore>,
        set: RefMut<JoinSet<Result<(), AppError>>>,
        publisher: Ref<Publisher>,
    ) -> Self {
        Self {
            semaphore,
            set,
            publisher,
        }
    }

    /// Add commands to be run when [execute] is called.
    pub fn add(&self, jobs: Vec<Job>) {
        for job in jobs {
            let id = job.get_id();
            let semaphore = self.semaphore.clone();
            let publisher = self.publisher.clone();
            publisher.update(&id, Created);
            let mut set = self.set.write().expect("join set to be writeable");
            set.spawn(async move {
                publisher.update(&id, Queued);
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should be available");
                publisher.update(&id, Started);
                job.execute().await?;
                publisher.update(&id, Completed);
                Ok(())
            });
        }
    }

    pub async fn execute(&self) -> Result<(), AppError> {
        self.publisher.start("");
        let mut set = self.set.write().expect("join set to be writeable");
        while let Some(result) = set.join_next().await {
            result.or_else(|e| AppError::task(e, "executing task"))??;
        }
        self.publisher.finish("");
        Ok(())
    }
}
