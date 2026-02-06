use crate::prelude::*;

/// Actions that can fail in the eyed3 module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum Eyed3Action {
    #[error("get details")]
    GetDetails,
}
