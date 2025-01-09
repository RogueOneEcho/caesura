use std::sync::Arc;

use crate::utils::*;

use di::{injectable, Ref, RefMut};
use rogue_logging::Error;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Execute a [Job] in parallel across a restricted number of threads.
///
/// [Semaphore] is used to limit the number of commands that can be executed concurrently.
/// [`JoinSet`] is used to execute commands in parallel, and collate the results.
/// [Publisher] is updated by an
/// [observer design pattern](https://refactoring.guru/design-patterns/observer) when the status
/// of a [Job] changes.
pub struct JobRunner {
    pub semaphore: Arc<Semaphore>,
    pub set: RefMut<JoinSet<Result<(), Error>>>,
    pub publisher: Ref<Publisher>,
}

#[injectable]
impl JobRunner {
    /// Create a new [`JobRunner`].
    pub fn new(
        semaphore: Arc<Semaphore>,
        set: RefMut<JoinSet<Result<(), Error>>>,
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

    /// Add commands to be run when [execute] is called.
    pub fn add_without_publish(&self, jobs: Vec<Job>) {
        for job in jobs {
            let semaphore = self.semaphore.clone();
            let mut set = self.set.write().expect("join set to be writeable");
            set.spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should be available");
                job.execute().await?;
                Ok(())
            });
        }
    }

    pub async fn execute(&self) -> Result<(), Error> {
        self.execute_internal(true).await
    }

    pub async fn execute_without_publish(&self) -> Result<(), Error> {
        self.execute_internal(false).await
    }

    async fn execute_internal(&self, publish: bool) -> Result<(), Error> {
        if publish {
            self.publisher.start("");
        }
        let mut set = self.set.write().expect("join set to be writeable");
        while let Some(result) = set.join_next().await {
            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    set.abort_all();
                    set.detach_all();
                    return Err(task_error(e, "executing task"));
                }
            };
            if let Err(e) = result {
                set.abort_all();
                set.detach_all();
                return Err(e);
            }
        }
        if publish {
            self.publisher.finish("");
        }
        Ok(())
    }
}
