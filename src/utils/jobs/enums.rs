use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Status {
    Created,
    Queued,
    Started,
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
