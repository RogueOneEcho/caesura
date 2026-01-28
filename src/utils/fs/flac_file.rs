use std::path::PathBuf;

use claxon::FlacReader;
use claxon::metadata::StreamInfo;
use lofty::tag::Tag;
use once_cell::sync::OnceCell;
use rogue_logging::Error;

use crate::utils::{DiscContext, convert_to_id3v2, fix_track_numbering, get_vorbis_tags};

/// A representation of a FLAC file.
pub struct FlacFile {
    /// Path to the file
    pub path: PathBuf,

    /// File name without the extension.
    pub file_name: String,

    /// Subdirectory of the file.
    pub sub_dir: PathBuf,

    /// Cached ID3 tags.
    ///
    /// Lazily loaded, converted from Vorbis. Uses thread-safe `OnceCell`.
    id3_tags: OnceCell<Tag>,

    /// Disc context for track renaming.
    ///
    /// Set once after collection
    pub disc_context: Option<DiscContext>,
}

impl FlacFile {
    #[must_use]
    pub fn new(path: PathBuf, source_dir: &PathBuf) -> Self {
        let sub_dir = path
            .strip_prefix(source_dir)
            .expect("Flac file path should start with the source directory")
            .parent()
            .expect("Flac file path should have a parent directory")
            .to_path_buf();
        let file_name = path
            .file_name()
            .expect("Flac file should have a name")
            .to_os_string()
            .to_string_lossy()
            .strip_suffix(".flac")
            .expect("Flac file should .flac extension")
            .to_owned();
        FlacFile {
            path,
            file_name,
            sub_dir,
            id3_tags: OnceCell::new(),
            disc_context: None,
        }
    }

    /// Get cached ID3 tags, converting from Vorbis and fixing track numbering.
    pub fn id3_tags(&self) -> Result<&Tag, Error> {
        self.id3_tags.get_or_try_init(|| {
            let mut tags = get_vorbis_tags(self)?;
            convert_to_id3v2(&mut tags);
            let _ = fix_track_numbering(&mut tags);
            Ok(tags)
        })
    }

    #[must_use]
    pub fn get_path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn get_stream_info(&self) -> Result<StreamInfo, claxon::Error> {
        let reader = FlacReader::open(&self.path)?;
        Ok(reader.streaminfo())
    }
}
