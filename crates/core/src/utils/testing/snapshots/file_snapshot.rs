use super::{AudioSnapshot, ImageSnapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Result;
use std::path::Path;

/// Snapshot of a single file for deterministic testing.
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

/// Extract the snapshot body (after the YAML front matter) from a `.snap` file.
fn read_snap_body(snap_path: &Path) -> Option<String> {
    let content = fs::read_to_string(snap_path).ok()?;
    content.splitn(3, "---\n").nth(2).map(String::from)
}

/// Patch platform-dependent fields from a stored insta snapshot.
///
/// On platforms where audio encoding produces bitwise-different output (ARM,
/// Nix), replaces SHA-256, file-size, and bitrate fields in `files` with values
/// from the stored snapshot so that structural comparison via
/// `assert_yaml_snapshot!` still works.
pub fn patch_platform_dependent_fields(files: &mut [FileSnapshot], snap_path: &Path) {
    let Some(yaml_body) = read_snap_body(snap_path) else {
        return;
    };
    let Ok(stored) = serde_yaml::from_str::<Vec<FileSnapshot>>(&yaml_body) else {
        return;
    };
    for (actual, stored) in files.iter_mut().zip(stored.iter()) {
        actual.sha256.clone_from(&stored.sha256);
        actual.size = stored.size;
        if let (Some(actual_audio), Some(stored_audio)) = (&mut actual.audio, &stored.audio) {
            actual_audio.overall_bitrate = stored_audio.overall_bitrate;
            actual_audio.audio_bitrate = stored_audio.audio_bitrate;
            for (actual_pic, stored_pic) in actual_audio
                .pictures
                .iter_mut()
                .zip(stored_audio.pictures.iter())
            {
                actual_pic.sha256.clone_from(&stored_pic.sha256);
                actual_pic.size = stored_pic.size;
            }
        }
    }
}

/// Assert that the actual output has the same number of lines as the stored snapshot.
///
/// Used for plain-text snapshots (e.g. inspect output) where exact matching is
/// not possible across platforms but structural shape should be preserved.
#[expect(clippy::panic, reason = "test assertion that should panic on failure")]
pub fn assert_line_count(actual: &str, snap_path: &Path) {
    let Some(stored) = read_snap_body(snap_path) else {
        panic!("stored snapshot not found at {}", snap_path.display());
    };
    let expected = stored.lines().count();
    let got = actual.lines().count();
    assert_eq!(
        got, expected,
        "inspect output should have {expected} lines but had {got}"
    );
}
