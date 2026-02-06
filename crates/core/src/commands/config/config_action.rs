use crate::prelude::*;

/// Actions that can fail in the config module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum ConfigAction {
    #[error("collate config")]
    CollateConfig,
    #[error("serialize config")]
    SerializeConfig,
}
