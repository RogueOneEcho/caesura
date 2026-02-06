use crate::prelude::*;
use std::sync::Arc;
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
    /// Semaphore to limit concurrent job execution.
    pub semaphore: Arc<Semaphore>,
    /// Set of spawned job tasks.
    pub set: RefMut<JoinSet<Result<(), Failure<JobAction>>>>,
    /// Publisher for job status updates.
    pub publisher: Ref<Publisher>,
}

#[injectable]
impl JobRunner {
    /// Create a new [`JobRunner`].
    pub fn new(
        semaphore: Arc<Semaphore>,
        set: RefMut<JoinSet<Result<(), Failure<JobAction>>>>,
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
                job.execute().await.map_err(|f| f.with("job", &id))?;
                publisher.update(&id, Completed);
                Ok(())
            });
        }
    }

    /// Add jobs to be run without publishing status updates.
    pub fn add_without_publish(&self, jobs: Vec<Job>) {
        for job in jobs {
            let id = job.get_id();
            let semaphore = self.semaphore.clone();
            let mut set = self.set.write().expect("join set to be writeable");
            set.spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should be available");
                job.execute().await.map_err(|f| f.with("job", &id))?;
                Ok(())
            });
        }
    }

    /// Execute all queued jobs and wait for completion.
    pub async fn execute(&self) -> Result<(), Failure<JobAction>> {
        self.execute_internal(true).await
    }

    /// Execute all queued jobs without publishing status updates.
    pub async fn execute_without_publish(&self) -> Result<(), Failure<JobAction>> {
        self.execute_internal(false).await
    }

    async fn execute_internal(&self, publish: bool) -> Result<(), Failure<JobAction>> {
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
                    return Err(Failure::new(JobAction::ExecuteTask, e));
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
