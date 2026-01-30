use crate::prelude::*;
use lofty::prelude::Accessor;

/// Context for disc/track information, computed once from all FLACs in a source.
#[derive(Debug, Clone)]
pub struct DiscContext {
    /// Dynamically calculated padding for track numbers.
    pub track_padding: usize,
    /// Whether the source contains multiple discs.
    pub is_multi_disc: bool,
    /// Number of discs in the source.
    #[allow(dead_code)]
    pub disc_count: u32,
}

impl DiscContext {
    /// Create a naming context by analyzing all FLAC files in a source.
    ///
    /// Finds global max track number across all discs and detects multi-disc releases.
    #[must_use]
    pub fn from_flacs(flacs: &[FlacFile]) -> Self {
        let mut max_track: u32 = 0;
        let mut disc_count: u32 = 1;
        for flac in flacs {
            if let Ok(tags) = flac.id3_tags() {
                if let Some(track) = tags.track() {
                    max_track = max_track.max(track);
                }
                disc_count = disc_count.max(tags.disk().unwrap_or(1));
            }
        }
        Self {
            track_padding: calculate_padding(max_track),
            is_multi_disc: disc_count > 1,
            disc_count,
        }
    }
}

/// Calculate padding width based on max track number.
///
/// Uses floor(log10(max)) + 1 to determine digit count.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions
)]
fn calculate_padding(max_track: u32) -> usize {
    if max_track == 0 {
        return 1;
    }
    (f64::from(max_track).log10().floor() as usize) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_single_digit() {
        assert_eq!(calculate_padding(1), 1);
        assert_eq!(calculate_padding(5), 1);
        assert_eq!(calculate_padding(9), 1);
    }

    #[test]
    fn padding_double_digit() {
        assert_eq!(calculate_padding(10), 2);
        assert_eq!(calculate_padding(50), 2);
        assert_eq!(calculate_padding(99), 2);
    }

    #[test]
    fn padding_triple_digit() {
        assert_eq!(calculate_padding(100), 3);
        assert_eq!(calculate_padding(500), 3);
        assert_eq!(calculate_padding(999), 3);
    }

    #[test]
    fn padding_four_digit() {
        assert_eq!(calculate_padding(1000), 4);
        assert_eq!(calculate_padding(5000), 4);
        assert_eq!(calculate_padding(9999), 4);
    }

    #[test]
    fn padding_zero_returns_one() {
        assert_eq!(calculate_padding(0), 1);
    }
}
