use crate::prelude::*;
use claxon::Error as ClaxonError;
use claxon::FlacReader;
use futures::StreamExt;
use futures::stream;

/// Verify FLAC files decode without error.
#[injectable]
pub(crate) struct DecodeVerifier {
    verify_options: Ref<VerifyOptions>,
    runner_options: Ref<RunnerOptions>,
}

impl DecodeVerifier {
    /// Decode every FLAC concurrently and return a [`SourceIssue`] for each that fails.
    ///
    /// - Skips entirely when `no_decode_test` is set
    /// - Runs up to `cpus` decodes at once on the blocking pool
    /// - Preserves input order via `buffered`, so issues need no sort
    /// - Cost scales with audio length as every frame is decoded, comparable to `flac --test`
    /// - Runs roughly 20x slower in debug builds than release due to unoptimized `claxon` decoding
    pub(crate) async fn execute(&self, flacs: &[FlacFile]) -> Vec<SourceIssue> {
        if self.verify_options.no_decode_check {
            debug!("{} decode check due to settings", "Skipped".bold());
            return Vec::new();
        }
        trace!("{} decode of {} FLACs", "Checking".bold(), flacs.len());
        let cpus = self.runner_options.get_cpus();
        let start = Instant::now();
        let issues: Vec<SourceIssue> = stream::iter(flacs.iter().map(|flac| flac.path.clone()))
            .map(|path| async move {
                spawn_blocking(move || decode_flac(&path))
                    .await
                    .expect("decode task should not panic")
            })
            .buffered(cpus)
            .filter_map(|issue| async move { issue })
            .collect()
            .await;
        trace!(
            "{} decode of {} FLACs in {:.3}s",
            "Checked".bold(),
            flacs.len(),
            start.elapsed().as_secs_f64()
        );
        issues
    }
}

/// Decode a single FLAC by path and return a [`SourceIssue`] if it fails.
pub(crate) fn decode_flac(path: &Path) -> Option<SourceIssue> {
    decode(path).err().map(|error| SourceIssue::DecodeError {
        path: path.to_path_buf(),
        error: format!("{error}"),
    })
}

/// Decode every audio frame of a FLAC file, discarding the samples.
///
/// - Validates each frame header CRC-8 and footer CRC-16 in process
/// - Surfaces truncation as a mid-frame decode error
/// - Reuses a single sample buffer across frames
fn decode(path: &Path) -> Result<(), ClaxonError> {
    trace!("Decoding FLAC {}", path.display());
    let mut reader = FlacReader::open(path)?;
    let mut buffer = Vec::new();
    let mut blocks = reader.blocks();
    while let Some(block) = blocks.read_next_or_eof(buffer)? {
        buffer = block.into_buffer();
    }
    Ok(())
}
