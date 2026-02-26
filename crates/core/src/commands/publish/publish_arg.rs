use crate::prelude::*;
use caesura_macros::Options;
use serde::{Deserialize, Serialize};

/// Path argument for the publish command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct PublishArg {
    /// Path to a publish YAML manifest file.
    #[arg(value_name = "PATH")]
    #[options(required)]
    pub publish_path: PathBuf,

    /// Validate and render upload payloads without uploading.
    #[arg(long)]
    pub dry_run: bool,
}

impl OptionsContract for PublishArg {
    type Partial = PublishArgPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if self.publish_path.as_os_str().is_empty() {
            errors.push(OptionRule::NotSet("Publish Path".to_owned()));
            return;
        }
        if !self.publish_path.is_file() {
            errors.push(OptionRule::DoesNotExist(
                "Publish Path".to_owned(),
                self.publish_path.to_string_lossy().to_string(),
            ));
        }
    }
}
