#[derive(strum_macros::Display, Clone)]
pub enum Status {
    Created,
    Queued,
    Started,
    Completed,
}
