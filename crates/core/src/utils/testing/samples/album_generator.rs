//! Generate test albums with configurable metadata and audio format.
//!
//! [`AlbumGenerator`] generates test albums as FLAC files with cover images and torrents.
//! It supports two modes:
//!
//! - **Ephemeral**: Generate in temp directories for isolated tests via [`AlbumGenerator::generate_in_dir`]
//! - **Cached**: Generate in `SAMPLE_SOURCES_DIR` with deduplication via [`AlbumProvider`]
//!
//! Use [`AlbumConfig`] to configure metadata (artist, album, tracks, disc numbers,
//! vinyl-style numbering) and audio format (bit depth, sample rate).

use crate::testing_prelude::*;

/// Generates a complete test album (FLAC files, cover image, torrent).
pub struct AlbumGenerator;

impl AlbumGenerator {
    /// Generate album in a specific directory without caching.
    ///
    /// Use this for isolated tests (e.g., determinism checks in temp directories).
    /// For cached generation, use [`AlbumProvider::generate`] instead.
    pub(super) async fn generate_in_dir(
        config: &AlbumConfig,
        source_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        Self::generate_files(config, source_dir).await
    }

    /// Generate album files in the specified directory.
    ///
    /// - Creates FLAC files, cover image, and torrent unconditionally
    /// - Caller is responsible for coordination (see [`AlbumProvider`])
    pub(super) async fn generate_files(
        config: &AlbumConfig,
        source_dir: &Path,
    ) -> Result<(), Failure<SampleAction>> {
        create_dir_all(source_dir).map_err(Failure::wrap(SampleAction::CreateDirectory))?;
        for track in &config.tracks {
            let mut generator = FlacGenerator::new()
                .with_filename(config.track_filename(track))
                .with_bit_depth(config.format.depth.as_u16())
                .with_sample_rate(config.format.rate.as_u32())
                .with_frequency(track.frequency)
                .with_artist(config.artist)
                .with_album(config.album)
                .with_title(track.title)
                .with_track_number(track.track_number)
                .with_date(config.year.to_string())
                .with_disc_number(track.disc_number)
                .with_cover_image();
            if let Some(subdir) = config.disc_subdir(track) {
                generator = generator.with_sub_directory(subdir);
            }
            if let Some(duration) = track.duration_secs {
                generator = generator.with_duration_secs(duration);
            }
            generator.generate(source_dir).await?;
        }
        ImageGenerator::new()
            .with_filename("cover.png")
            .generate(source_dir)?;
        let torrent_path = append_extension(source_dir, "torrent");
        TorrentCreator::create(
            source_dir,
            &torrent_path,
            format!("{RED_TRACKER_URL}/test/announce"),
            Indexer::Red,
        )
        .await
        .map_err(Failure::wrap(SampleAction::CreateTorrent))?;
        Ok(())
    }
}

/// Append an extension to a path.
///
/// Unlike [`Path::with_extension`], this appends rather than replaces.
/// Needed for paths containing dots like `{16-44.1} (FLAC)`.
fn append_extension(path: &Path, ext: &str) -> PathBuf {
    let mut result = path.as_os_str().to_os_string();
    result.push(".");
    result.push(ext);
    PathBuf::from(result)
}
