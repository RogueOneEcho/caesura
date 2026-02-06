use std::path::{Path, PathBuf};

use image::{Rgb, RgbImage};
use rogue_logging::Failure;

use super::SampleAction;

/// Builder for generating sample PNG images with a gradient pattern.
///
/// # Example
/// ```ignore
/// ImageGenerator::new()
///     .with_filename("cover.png")
///     .with_size(100, 100)
///     .generate(&output_dir)?;
/// ```
#[derive(Debug, Clone)]
pub struct ImageGenerator {
    width: u32,
    height: u32,
    filename: String,
}

impl Default for ImageGenerator {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            filename: "image.png".to_owned(),
        }
    }
}

impl ImageGenerator {
    /// Create a new image generator with default parameters.
    ///
    /// Defaults:
    /// - Size: 100x100
    /// - Filename: "image.png"
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the image dimensions (default: 100x100).
    #[must_use]
    #[expect(dead_code)]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the output filename (default: "image.png").
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = filename.into();
        self
    }

    /// Generate the image in the specified output directory.
    ///
    /// Returns the full path to the generated file.
    pub fn generate(&self, output_dir: &Path) -> Result<PathBuf, Failure<SampleAction>> {
        let path = output_dir.join(&self.filename);
        let mut img = RgbImage::new(self.width, self.height);
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::integer_division
        )]
        for y in 0..self.height {
            for x in 0..self.width {
                // Red decreases left-to-right, blue increases top-to-bottom
                let r = (255 - (x * 255 / self.width.max(1))) as u8;
                let b = (y * 255 / self.height.max(1)) as u8;
                img.put_pixel(x, y, Rgb([r, 0, b]));
            }
        }
        img.save(&path)
            .map_err(Failure::wrap(SampleAction::SaveImage))?;
        Ok(path)
    }
}
