/// Metadata for a single option field for documentation generation.
#[derive(Debug, Clone)]
pub struct FieldDoc {
    /// Config key in `snake_case` (e.g., "`wait_before_upload`")
    pub config_key: &'static str,
    /// CLI flag in kebab-case (e.g., "--wait-before-upload")
    pub cli_flag: &'static str,
    /// Type display string (e.g., "bool", "u32", "Option<String>")
    pub field_type: &'static str,
    /// Default value as string, if any
    pub default: Option<&'static str>,
    /// Description extracted from doc comments
    pub description: &'static str,
}

/// Metadata for an options struct for documentation generation.
#[derive(Debug, Clone)]
pub struct OptionsDoc {
    /// Name of the options struct
    pub name: &'static str,
    /// Description extracted from struct doc comments
    pub description: &'static str,
    /// Documentation for each field
    pub fields: &'static [FieldDoc],
    /// Commands that use this options struct
    pub commands: &'static [&'static str],
}

/// Trait for options structs to provide documentation metadata.
pub trait Documented {
    /// Returns the documentation metadata for this options struct.
    fn doc_metadata() -> &'static OptionsDoc;
}
