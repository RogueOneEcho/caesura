use crate::prelude::*;
use claxon::Error as ClaxonError;
use claxon::FlacReader;

/// Verify FLAC files decode without error.
#[injectable]
pub(crate) struct DecodeVerifier {
    verify_options: Ref<VerifyOptions>,
}

impl DecodeVerifier {
    /// Decode every FLAC and return a [`SourceIssue`] for each that fails.
    ///
    /// - Skips entirely when `no_decode_test` is set
    /// - Cost scales with audio length as every frame is decoded, comparable to `flac --test`
    /// - Runs roughly 20x slower in debug builds than release due to unoptimized `claxon` decoding
    pub(crate) fn execute(&self, flacs: &[FlacFile]) -> Vec<SourceIssue> {
        if self.verify_options.no_decode_test {
            return Vec::new();
        }
        let start = Instant::now();
        let issues: Vec<SourceIssue> = flacs
            .iter()
            .filter_map(|flac| decode_flac(&flac.path))
            .collect();
        trace!(
            "Decode tested {} FLACs in {:.3}s",
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
    let mut reader = FlacReader::open(path)?;
    let mut buffer = Vec::new();
    let mut blocks = reader.blocks();
    while let Some(block) = blocks.read_next_or_eof(buffer)? {
        buffer = block.into_buffer();
    }
    Ok(())
}
