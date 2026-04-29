use crate::prelude::*;
use claxon::Error as ClaxonError;
use claxon::FlacReader;
use claxon::metadata::StreamInfo;
use lofty::id3::v2::Id3v2Tag;
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

    /// Cached raw Vorbis tags.
    ///
    /// Lazily loaded. Uses thread-safe `OnceCell`.
    vorbis_tags: OnceCell<Tag>,

    /// Cached ID3 tags.
    ///
    /// Lazily loaded. Uses thread-safe `OnceCell`.
    id3_tags: OnceCell<Id3v2Tag>,

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
            vorbis_tags: OnceCell::new(),
            id3_tags: OnceCell::new(),
            disc_context: None,
        }
    }

    /// Get cached raw Vorbis tags without any conversion.
    pub fn vorbis_tags(&self) -> Result<&Tag, Failure<TranscodeAction>> {
        self.vorbis_tags.get_or_try_init(|| {
            get_vorbis_tags(self).map_err(Failure::wrap(TranscodeAction::GetTags))
        })
    }

    /// Get cached ID3 tags, round-tripped through [`Id3v2Tag`] conversion.
    ///
    /// Values that cannot be represented in `ID3v2` format (e.g. non-numeric
    /// track numbers) are dropped during the round-trip, matching the
    /// behavior of [`save_id3v2_deterministic`].
    pub fn id3_tags(&self) -> Result<&Id3v2Tag, Failure<TranscodeAction>> {
        self.id3_tags.get_or_try_init(|| {
            let mut tags = self.vorbis_tags()?.clone();
            fix_track_numbering(&mut tags);
            Ok(Id3v2Tag::from(tags))
        })
    }

    /// Full path as a string.
    #[must_use]
    pub fn get_path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    /// FLAC stream info containing sample rate, channels, and bit depth.
    pub fn get_stream_info(&self) -> Result<StreamInfo, ClaxonError> {
        let reader = FlacReader::open(&self.path)?;
        Ok(reader.streaminfo())
    }
}
