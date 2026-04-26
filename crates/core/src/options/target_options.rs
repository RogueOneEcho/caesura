use crate::prelude::*;

/// Options for transcoding
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct TargetOptions {
    /// Formats to attempt to transcode to.
    #[arg(long)]
    #[options(default = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0])]
    pub target: Vec<TargetFormat>,

    /// Allow transcoding to existing formats.
    ///
    /// Note: This is only useful for development and should probably not be used.
    #[arg(long)]
    pub allow_existing: bool,

    /// Allow transcoding when the source has empty edition fields but an existing torrent does not.
    #[arg(long)]
    pub allow_less_specific: bool,

    /// Use random dithering when resampling with `SoX`.
    ///
    /// By default, `SoX` runs in repeatable mode (`-R`) which seeds the dither
    /// random number generator with a fixed value, producing deterministic output.
    /// Set this to `true` to use random dithering instead.
    #[arg(long)]
    pub sox_random_dither: bool,

    /// Vorbis comment tag names to exclude from transcoded output.
    #[arg(long)]
    #[options(default = TargetOptions::default_exclude_vorbis_comments())]
    pub exclude_vorbis_comments: Vec<String>,
}

impl TargetOptions {
    /// Default Vorbis comment tag names to exclude from transcoded output.
    #[must_use]
    pub fn default_exclude_vorbis_comments() -> Vec<String> {
        vec![
            "COMMENT".to_owned(),
            "ENCODER".to_owned(),
            "RATING".to_owned(),
            "WORK".to_owned(),
        ]
    }
}

impl OptionsContract for TargetOptions {
    type Partial = TargetOptionsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        validator.check_non_empty("target", &self.target);
    }
}
