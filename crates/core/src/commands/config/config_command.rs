use crate::prelude::*;

/// Display the current configuration options.
#[injectable]
pub struct ConfigCommand {
    batch_options: Ref<BatchOptions>,
    cache_options: Ref<CacheOptions>,
    config_options: Ref<ConfigOptions>,
    copy_options: Ref<CopyOptions>,
    cross_config_options: Ref<CrossConfigOptions>,
    cross_options: Ref<CrossOptions>,
    file_options: Ref<FileOptions>,
    queue_add_args: Ref<QueueAddArgs>,
    runner_options: Ref<RunnerOptions>,
    shared_options: Ref<SharedOptions>,
    sox_options: Ref<SoxOptions>,
    spectrogram_options: Ref<SpectrogramOptions>,
    target_options: Ref<TargetOptions>,
    upload_options: Ref<UploadOptions>,
    verify_options: Ref<VerifyOptions>,
}

impl ConfigCommand {
    /// Execute the config command, printing documented YAML to stdout.
    pub fn execute(&self) -> Result<bool, Failure<ConfigAction>> {
        let output = self.render()?;
        print!("{output}");
        Ok(true)
    }

    /// Render all resolved options as documented YAML.
    pub fn render(&self) -> Result<String, Failure<ConfigAction>> {
        let options = self
            .get_options_map()
            .map_err(Failure::wrap(ConfigAction::CollateConfig))?;
        let docs = Self::get_docs_map();
        let mut out = String::new();
        for (key, value) in options {
            if let Some(field_doc) = docs.get(key.as_str()) {
                for line in field_doc.description.split("<br>") {
                    let _ = writeln!(out, "{}", format_comment(format!("# {line}")));
                }
                let default = field_doc
                    .default_doc
                    .map(String::from)
                    .or_else(|| field_doc.default_value.clone());
                if let Some(default) = default {
                    let _ = writeln!(out, "{}", format_comment(format!("# Default: {default}")));
                }
            }
            let yaml =
                yaml_to_string(&value).map_err(Failure::wrap(ConfigAction::SerializeValue))?;
            let yaml = yaml.trim_end_matches('\n');
            let is_block = matches!(value, Value::Sequence(s) if !s.is_empty());
            let spacer = if is_block { '\n' } else { ' ' };
            let _ = writeln!(out, "{}:{spacer}{}", format_key(&key), format_value(yaml));
        }
        Ok(out)
    }

    fn get_options_map(&self) -> Result<BTreeMap<String, Value>, YamlError> {
        let option_values = [
            yaml_to_value(&*self.batch_options)?,
            yaml_to_value(&*self.cache_options)?,
            yaml_to_value(&*self.config_options)?,
            yaml_to_value(&*self.copy_options)?,
            yaml_to_value(&*self.cross_config_options)?,
            yaml_to_value(&*self.cross_options)?,
            yaml_to_value(&*self.file_options)?,
            yaml_to_value(&*self.queue_add_args)?,
            yaml_to_value(&*self.runner_options)?,
            yaml_to_value(&*self.shared_options)?,
            yaml_to_value(&*self.sox_options)?,
            yaml_to_value(&*self.spectrogram_options)?,
            yaml_to_value(&*self.target_options)?,
            yaml_to_value(&*self.upload_options)?,
            yaml_to_value(&*self.verify_options)?,
        ];
        let mut data = BTreeMap::new();
        for value in option_values {
            if let Value::Mapping(map) = value {
                for (k, v) in map {
                    if let Value::String(key) = k {
                        data.insert(key, v);
                    }
                }
            }
        }
        Ok(data)
    }

    fn get_docs_map() -> HashMap<&'static str, &'static FieldDoc> {
        let mut map = HashMap::new();
        for doc in OptionsRegistration::get_all() {
            for field in &doc.fields {
                map.insert(field.config_key, field);
            }
        }
        map
    }
}

fn format_comment(input: String) -> ColoredString {
    input.green().dimmed()
}

fn format_key(input: &str) -> ColoredString {
    input.yellow()
}

fn format_value(input: &str) -> &str {
    input
}
