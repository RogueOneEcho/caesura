use crate::prelude::*;

/// Actions that can fail in the metaflac module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum MetaflacAction {
    #[error("get details")]
    GetDetails,
}
