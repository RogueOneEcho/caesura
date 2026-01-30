use serde::{Deserialize, Serialize};

use crate::options::OptionRule;
use caesura_macros::Options;

/// Options for image resizing
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
#[options(commands(Batch, Transcode))]
#[options(field_name = "file")]
pub struct FileOptions {
    /// Should compression of images be disabled?
    ///
    /// Default: `false`
    #[arg(long)]
    pub no_image_compression: bool,

    /// Should transcoded files be renamed from source filenames to a
    /// standardized format: `{track:0>N} {title}.{ext}`?
    ///
    /// Multi-disc releases will be organized into `CD1/`, `CD2/` subfolders.
    ///
    /// Default: `false`
    #[arg(long)]
    pub rename_tracks: bool,

    /// Maximum file size in bytes beyond which images are compressed.
    ///
    /// Default: `750000`
    ///
    /// Only applies to image files.
    #[arg(long)]
    #[options(default = 750_000)]
    pub max_file_size: u64,

    /// Maximum size in pixels for images
    ///
    /// Default: `1280`
    ///
    /// Only applied if the image is greater than `max_file_size`.
    #[arg(long)]
    #[options(default = 1280)]
    pub max_pixel_size: u32,

    /// Quality percentage to apply for jpg compression.
    ///
    /// Default: `80`
    ///
    /// Only applied if the image is greated than `max_file_size`.
    #[arg(long)]
    #[options(default = 80)]
    pub jpg_quality: u8,

    /// Should conversion of png images to jpg be disabled?
    ///
    /// Default: `false`
    ///
    /// Only applied if the image is greater than `max_file_size`.
    #[arg(long)]
    pub no_png_to_jpg: bool,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            no_image_compression: false,
            rename_tracks: false,
            max_file_size: 750_000,
            max_pixel_size: 1280,
            jpg_quality: 80,
            no_png_to_jpg: false,
        }
    }
}

impl FileOptions {
    /// Validate the partial options.
    pub fn validate_partial(_: &FileOptionsPartial, _: &mut Vec<OptionRule>) {}
}
