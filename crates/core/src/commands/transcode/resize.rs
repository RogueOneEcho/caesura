use crate::prelude::*;

/// Information needed to resize an image
pub(crate) struct Resize {
    /// Path to the input file
    pub input: PathBuf,
    /// Path to the output file
    pub output: PathBuf,
    /// Maximum size in pixels
    pub max_pixel_size: u32,
    /// Quality percentage to apply for jpg compression.
    pub quality: u8,
}

impl Resize {
    /// Create a new convert command.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_info(self) -> CommandInfo {
        CommandInfo {
            program: CONVERT.to_owned(),
            args: vec![
                self.input.to_string_lossy().to_string(),
                "-resize".to_owned(),
                format!("{}x{}>", self.max_pixel_size, self.max_pixel_size),
                "-quality".to_owned(),
                format!("{}%", self.quality),
                self.output.to_string_lossy().to_string(),
            ],
        }
    }
}
