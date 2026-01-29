use std::fmt::{Display, Formatter};

/// Execution status of a [`Job`].
#[derive(Clone)]
pub enum Status {
    /// Job created but not yet queued.
    Created,
    /// Job queued and waiting for a permit.
    Queued,
    /// Job acquired a permit and is executing.
    Started,
    /// Job finished execution.
    Completed,
}

impl Display for Status {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Created => write!(formatter, "Created"),
            Status::Queued => write!(formatter, "Queued"),
            Status::Started => write!(formatter, "Started"),
            Status::Completed => write!(formatter, "Completed"),
        }
    }
}
