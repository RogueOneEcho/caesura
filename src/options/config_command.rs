use crate::errors::AppError;
use crate::options::{
    BatchOptions, FileOptions, SharedOptions, SpectrogramOptions, TargetOptions, UploadOptions,
    VerifyOptions,
};
use di::{injectable, Ref};
use serde_json::Value;
use std::collections::BTreeMap;

/// Config a FLAC source is suitable for transcoding.
#[allow(clippy::struct_field_names)]
#[injectable]
pub struct ConfigCommand {
    batch_options: Ref<BatchOptions>,
    file_options: Ref<FileOptions>,
    shared_options: Ref<SharedOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    target_options: Ref<TargetOptions>,
    upload_options: Ref<UploadOptions>,
    verify_options: Ref<VerifyOptions>,
}

impl ConfigCommand {
    pub fn execute(&self) -> Result<bool, AppError> {
        let options = self
            .get_options_hashmap()
            .or_else(|e| AppError::deserialization(e, "collate config"))?;
        let json = serde_json::to_string_pretty(&options)
            .or_else(|e| AppError::deserialization(e, "serialize config"))?;
        println!("{json}");
        Ok(true)
    }

    fn get_options_hashmap(&self) -> Result<BTreeMap<String, Value>, serde_json::Error> {
        let options = [
            serde_json::to_value(&*self.batch_options)?,
            serde_json::to_value(&*self.file_options)?,
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
