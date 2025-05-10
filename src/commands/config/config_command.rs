use crate::options::*;
use crate::utils::*;

use di::{Ref, injectable};
use rogue_logging::Error;
use serde_json::Value;
use std::collections::BTreeMap;

/// Config a FLAC source is suitable for transcoding.
#[allow(clippy::struct_field_names)]
#[injectable]
pub struct ConfigCommand {
    batch_options: Ref<BatchOptions>,
    cache_options: Ref<CacheOptions>,
    file_options: Ref<FileOptions>,
    runner_options: Ref<RunnerOptions>,
    shared_options: Ref<SharedOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    target_options: Ref<TargetOptions>,
    upload_options: Ref<UploadOptions>,
    verify_options: Ref<VerifyOptions>,
}

impl ConfigCommand {
    pub fn execute(&self) -> Result<bool, Error> {
        let options = self
            .get_options_hashmap()
            .map_err(|e| json_error(e, "collate config"))?;
        let yaml =
            serde_yaml::to_string(&options).map_err(|e| yaml_error(e, "serialize config"))?;
        println!("{yaml}");
        Ok(true)
    }

    fn get_options_hashmap(&self) -> Result<BTreeMap<String, Value>, serde_json::Error> {
        let options = [
            serde_json::to_value(&*self.batch_options)?,
            serde_json::to_value(&*self.cache_options)?,
            serde_json::to_value(&*self.file_options)?,
            serde_json::to_value(&*self.runner_options)?,
            serde_json::to_value(&*self.shared_options)?,
            serde_json::to_value(&*self.spectrogram_options)?,
            serde_json::to_value(&*self.target_options)?,
            serde_json::to_value(&*self.upload_options)?,
            serde_json::to_value(&*self.verify_options)?,
        ];
        let mut data: BTreeMap<String, Value> = BTreeMap::new();
        for option in &options {
            if let Some(map) = option.as_object() {
                data.extend(map.clone());
            }
        }
        Ok(data)
    }
}
