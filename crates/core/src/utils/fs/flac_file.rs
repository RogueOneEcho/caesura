use crate::prelude::*;
use crate::utils::{convert_to_id3v2, fix_track_numbering, get_vorbis_tags};
use claxon::FlacReader;
use claxon::metadata::StreamInfo;
use lofty::tag::Tag;
use once_cell::sync::OnceCell;

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
    /// Create a new [`FlacFile`] from a path.
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
    pub fn id3_tags(&self) -> Result<&Tag, Failure<TranscodeAction>> {
        self.id3_tags.get_or_try_init(|| {
            let mut tags =
                get_vorbis_tags(self).map_err(Failure::wrap(TranscodeAction::GetTags))?;
            convert_to_id3v2(&mut tags);
            let _ = fix_track_numbering(&mut tags);
            Ok(tags)
        })
    }

    /// Full path as a string.
    #[must_use]
    pub fn get_path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    /// FLAC stream info containing sample rate, channels, and bit depth.
    pub fn get_stream_info(&self) -> Result<StreamInfo, claxon::Error> {
        let reader = FlacReader::open(&self.path)?;
        Ok(reader.streaminfo())
    }
}
