use image::ImageReader;
use serde::Serialize;
use std::path::Path;

/// Snapshot of image metadata for deterministic testing.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageSnapshot {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Color type (e.g., `Rgb8`, `Rgba8`).
    pub color_type: String,
}

impl ImageSnapshot {
    /// Create an [`ImageSnapshot`] from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let img = ImageReader::open(path).ok()?.decode().ok()?;
        Some(Self {
            width: img.width(),
            height: img.height(),
            color_type: format!("{:?}", img.color()),
        })
    }
}
