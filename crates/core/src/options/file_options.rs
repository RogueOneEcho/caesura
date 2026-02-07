use serde::{Deserialize, Serialize};

use crate::commands::CommandArguments::{self, *};
use crate::options::{FromArgs, OptionRule, OptionsContract};
use caesura_macros::Options;

/// Options for image resizing
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct FileOptions {
    /// Should compression of images be disabled?
    #[arg(long)]
    pub no_image_compression: bool,

    /// Should transcoded files be renamed from source filenames to a
    /// standardized format: `{track:0>N} {title}.{ext}`?
    ///
    /// Multi-disc releases will be organized into `CD1/`, `CD2/` subfolders.
    #[arg(long)]
    pub rename_tracks: bool,

    /// Maximum file size in bytes beyond which images are compressed.
    ///
    /// Only applies to image files.
    #[arg(long)]
    #[options(default = 750_000)]
    pub max_file_size: u64,

    /// Maximum size in pixels for images.
    ///
    /// Only applied if the image is greater than `max_file_size`.
    #[arg(long)]
    #[options(default = 1280)]
    pub max_pixel_size: u32,

    /// Quality percentage to apply for jpg compression.
    ///
    /// Only applied if the image is greater than `max_file_size`.
    #[arg(long)]
    #[options(default = 80)]
    pub jpg_quality: u8,

    /// Should conversion of png images to jpg be disabled?
    ///
    /// Only applied if the image is greater than `max_file_size`.
    #[arg(long)]
    pub no_png_to_jpg: bool,
}

impl FileOptions {
    /// Default maximum file size in bytes beyond which images are compressed.
    pub const DEFAULT_MAX_FILE_SIZE: u64 = 750_000;
    /// Default maximum size in pixels for images.
    pub const DEFAULT_MAX_PIXEL_SIZE: u32 = 1280;
    /// Default quality percentage for JPG compression.
    pub const DEFAULT_JPG_QUALITY: u8 = 80;
}

impl OptionsContract for FileOptions {
    type Partial = FileOptionsPartial;
    fn validate(&self, _errors: &mut Vec<OptionRule>) {}
}

impl FromArgs for FileOptionsPartial {
    fn from_args(args: &Option<CommandArguments>) -> Option<Self> {
        match args {
            Some(Batch { file, .. } | Transcode { file, .. }) => Some(file.clone()),
            _ => None,
        }
    }
}
