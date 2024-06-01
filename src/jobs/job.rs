use crate::errors::AppError;
use crate::spectrogram::SpectrogramJob;
use crate::transcode::{AdditionalJob, TranscodeJob};

/// A job is a stand-alone object that contains all the information needed to perform
/// a piece of work.
///
/// Effectively it's a [command design pattern](https://refactoring.guru/design-patterns/command)
/// but in Command in Rust specifically refers to executing an external program so the term Job
/// will suffice.
///
/// Jobs can be executed by themselves, but they're intended to be executed in parallel
/// by a [`JobRunner`].
///
/// In theory, they could produce a result but the implement here is `Result<()>`.
pub enum Job {
    Additional(AdditionalJob),
    Spectrogram(SpectrogramJob),
    Transcode(TranscodeJob),
}

/// A command that can be executed in parallel.
///
/// A [command design pattern](https://refactoring.guru/design-patterns/command) is used
/// so the execution of the command can be deferred and multiple commands can be executed
/// in parallel via the multithreaded [`CommandRunner`].
impl Job {
    /// Get the ID of the wrapped command.
    #[must_use]
    pub fn get_id(&self) -> String {
        match self {
            Job::Additional(job) => job.id.clone(),
            Job::Spectrogram(job) => job.id.clone(),
            Job::Transcode(job) => job.id.clone(),
        }
    }

    /// Execute the wrapped command.
    pub async fn execute(self) -> Result<(), AppError> {
        match self {
            Job::Additional(job) => job.execute().await,
            Job::Spectrogram(job) => job.execute().await,
            Job::Transcode(job) => job.execute().await,
        }
    }
}
