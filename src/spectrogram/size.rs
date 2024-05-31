use clap::ValueEnum;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    Full,
    Zoom,
}
