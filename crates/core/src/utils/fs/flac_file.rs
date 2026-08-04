use crate::prelude::*;
use claxon::Error as ClaxonError;
use claxon::FlacReader;
use claxon::metadata::StreamInfo;
use lofty::id3::v2::Id3v2Tag;
use lofty::tag::Tag;
use once_cell::sync::OnceCell;

/// A representation of a FLAC file.
pub struct FlacFile {
    /// Path to the existing file on disk
    ///
    /// - May contain invalid UTF-8 characters or decomposed (NFD) sequences
    pub path: PathBuf,

    /// Output file name without extension
    ///
    /// - Invalid UTF-8 is replaced with `�`
    /// - Decomposed (NFD) sequences are replaced with composed (NFC)
    pub file_name: String,

    /// Output subdirectory, relative to the source directory
    ///
    /// - Invalid UTF-8 is replaced with `�`
    /// - Decomposed (NFD) sequences are replaced with composed (NFC)
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
            .to_nfc();
        let file_name = path
            .file_stem()
            .expect("Flac file should have a name")
            .to_string_lossy()
            .to_nfc();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Lowercase `.flac` extension is stripped from the file name.
    #[test]
    fn flac_file_new_lowercase() {
        // Arrange
        let source_dir = PathBuf::from("/music");
        let path = PathBuf::from("/music/01. Track.flac");
        // Act
        let flac = FlacFile::new(path, &source_dir);
        // Assert
        assert_eq!(flac.file_name, "01. Track");
    }

    /// Uppercase `.FLAC` extension is stripped from the file name.
    #[test]
    fn flac_file_new_uppercase() {
        // Arrange
        let source_dir = PathBuf::from("/music");
        let path = PathBuf::from("/music/01. Track.FLAC");
        // Act
        let flac = FlacFile::new(path, &source_dir);
        // Assert
        assert_eq!(flac.file_name, "01. Track");
    }

    /// Decomposed sequences in the file name and subdirectory are composed.
    #[test]
    fn flac_file_new_decomposed() {
        // Arrange
        let source_dir = PathBuf::from("/music");
        let path = PathBuf::from("/music/No\u{308}rd/01. Cafe\u{301}.flac");
        // Act
        let flac = FlacFile::new(path, &source_dir);
        // Assert
        assert_eq!(flac.file_name, "01. Caf\u{e9}");
        assert_eq!(flac.sub_dir, PathBuf::from("N\u{f6}rd"));
    }
}
