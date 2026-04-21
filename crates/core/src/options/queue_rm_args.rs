use crate::prelude::*;

/// Options for the [`QueueRemoveCommand`]
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct QueueRemoveArgs {
    /// A torrent hash
    #[arg(value_name = "HASH")]
    pub queue_rm_hash: String,
}

impl OptionsContract for QueueRemoveArgs {
    type Partial = QueueRemoveArgsPartial;

    fn validate(&self, errors: &mut Vec<OptionRule>) {
        if Hash::<20>::from_string(&self.queue_rm_hash).is_err() {
            errors.push(OptionRule::HashInvalid(
                "Queue remove hash".to_owned(),
                self.queue_rm_hash.clone(),
            ));
        }
    }
}

impl Display for QueueRemoveArgs {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let output = if let Ok(yaml) = yaml_to_string(self) {
            yaml
        } else {
            format!("{self:?}")
        };
        write!(formatter, "{output}")
    }
}
