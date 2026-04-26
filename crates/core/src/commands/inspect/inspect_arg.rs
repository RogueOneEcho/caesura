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

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_dir_exists("inspect_path", &self.inspect_path);
    }
}
