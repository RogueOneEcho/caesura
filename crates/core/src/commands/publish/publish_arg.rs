use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Path argument for the publish command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct PublishArg {
    /// Path to a publish YAML manifest file.
    #[arg(value_name = "PATH")]
    pub publish_path: PathBuf,
}

impl OptionsContract for PublishArg {
    type Partial = PublishArgPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if !self.publish_path.is_file() {
            errors.push(OptionRule::DoesNotExist(
                "Publish Path".to_owned(),
                self.publish_path.to_string_lossy().to_string(),
            ));
        }
    }
}
