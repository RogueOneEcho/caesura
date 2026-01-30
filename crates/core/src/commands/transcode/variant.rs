use crate::prelude::*;

/// Transcode operation variant.
pub enum Variant {
    /// Decode FLAC and encode to MP3.
    Transcode(Decode, Encode),
    /// Resample high-resolution FLAC to 16-bit.
    Resample(Resample),
    /// Copy or hard-link FLAC that needs no conversion.
    Include(Include),
}
