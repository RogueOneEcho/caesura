use super::{AudioSnapshot, ImageSnapshot};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Result;
use std::path::Path;

/// Snapshot of a single file for deterministic testing.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileSnapshot {
    /// Directory path relative to the snapshot root, if not in root.
    pub directory: Option<String>,
    /// Filename including extension.
    pub filename: String,
    /// File size in bytes.
    pub size: u64,
    /// SHA-256 hash of file contents.
    pub sha256: String,
    /// Audio metadata if the file is an audio file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioSnapshot>,
    /// Image metadata if the file is an image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageSnapshot>,
}

impl FileSnapshot {
    /// Create a [`FileSnapshot`] from a file path relative to a root directory.
    pub fn from_path(root: &Path, path: &Path) -> Result<Self> {
        let relative = path.strip_prefix(root).expect("path should be under root");

        let directory = relative.parent().and_then(|p| {
            let s = p.to_string_lossy().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

        let filename = relative
            .file_name()
            .expect("path should have filename")
            .to_string_lossy()
            .to_string();

        let ext = get_extension(path);
        let content = fs::read(path)?;
        let size = u64::try_from(content.len()).expect("file size fits in u64");

        Ok(Self {
            directory,
            filename,
            size,
            sha256: format!("{:x}", Sha256::digest(&content)),
            audio: parse_audio_metadata(path, ext.as_deref()),
            image: parse_image_metadata(path, ext.as_deref()),
        })
    }
}

fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
}

fn parse_audio_metadata(path: &Path, ext: Option<&str>) -> Option<AudioSnapshot> {
    match ext {
        Some("mp3" | "flac") => AudioSnapshot::from_path(path),
        _ => None,
    }
}

fn parse_image_metadata(path: &Path, ext: Option<&str>) -> Option<ImageSnapshot> {
    match ext {
        Some("png" | "jpg" | "jpeg" | "gif" | "webp") => ImageSnapshot::from_path(path),
        _ => None,
    }
}
