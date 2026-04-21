//! Options framework: traits, providers, and registration infrastructure.

pub use args_provider::*;
pub use command_trait::*;
pub use doc_metadata::*;
pub use options_provider::*;
pub use options_registration::*;
pub use options_trait::*;
pub use rules::*;

mod args_provider;
mod command_trait;
mod doc_metadata;
mod options_provider;
mod options_registration;
mod options_trait;
mod rules;
