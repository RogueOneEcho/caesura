use crate::prelude::*;
use claxon::Error as ClaxonError;
use claxon::FlacReader;

/// Verify a FLAC file decodes without error.
pub(crate) struct DecodeVerifier;

impl DecodeVerifier {
    /// Fully decode a FLAC file and return a [`SourceIssue`] if decoding fails.
    ///
    /// - Cost scales with audio length as every frame is decoded, comparable to `flac --test`
    /// - Runs roughly 20x slower in debug builds than release due to unoptimized `claxon` decoding
    pub(crate) fn execute(flac: &FlacFile) -> Option<SourceIssue> {
        decode(&flac.path)
            .err()
            .map(|error| SourceIssue::DecodeError {
                path: flac.path.clone(),
                error: format!("{error}"),
            })
    }
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
