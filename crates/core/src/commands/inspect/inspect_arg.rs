use crate::prelude::*;

/// Path argument for the inspect command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct InspectArg {
    /// Path to directory containing audio files.
    #[arg(value_name = "PATH")]
    pub inspect_path: PathBuf,
}

impl OptionsContract for InspectArg {
    type Partial = InspectArgPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if !self.inspect_path.exists() {
            errors.push(OptionRule::DoesNotExist(
                "inspect_path".to_owned(),
                self.inspect_path.to_string_lossy().to_string(),
            ));
        }
    }
}
