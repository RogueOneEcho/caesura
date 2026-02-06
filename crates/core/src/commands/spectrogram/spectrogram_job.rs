use crate::prelude::*;
use std::fs::create_dir_all;
use std::process::Output;
use tokio::process::Command;

/// Duration of the zoom spectrogram capture window in seconds.
const ZOOM_DURATION: u32 = 2;

/// Standard start position for zoom spectrogram in seconds (1:00).
const TYPICAL_ZOOM_START: u32 = 60;

/// A command to generate a spectrogram image of a FLAC file using sox.
///
/// A [command design pattern](https://refactoring.guru/design-patterns/command) is used
/// so the execution of the command can be deferred and multiple commands can be executed
/// in parallel via the multithreaded [`SpectrogramCommandRunner`].
pub(crate) struct SpectrogramJob {
    /// Job identifier for progress tracking.
    pub id: String,
    /// Path to the source FLAC file.
    pub source_path: String,
    /// Path to write the spectrogram image.
    pub output_path: PathBuf,
    /// Title to embed in the spectrogram image.
    pub image_title: String,
    /// Spectrogram size variant.
    pub size: Size,
    pub duration_secs: Option<u32>,
}

impl SpectrogramJob {
    /// Execute the command to generate the spectrogram.
    pub(crate) async fn execute(self) -> Result<(), Failure<SpectrogramAction>> {
        let output_dir = self
            .output_path
            .parent()
            .expect("output path should have a parent");
        create_dir_all(output_dir).map_err(Failure::wrap_with_path(
            SpectrogramAction::CreateOutputDirectory,
            output_dir,
        ))?;
        match self.size {
            Size::Full => self.execute_full().await,
            Size::Zoom => self.execute_zoom().await,
        }?;
        Ok(())
    }

    async fn execute_zoom(&self) -> Result<Output, Failure<SpectrogramAction>> {
        let start_time = calculate_zoom_start(self.duration_secs);
        let duration = format!("0:{ZOOM_DURATION:02}");
        Command::new(SOX)
            .arg(&self.source_path)
            .arg("-n")
            .arg("remix")
            .arg("1")
            .arg("spectrogram")
            .arg("-x")
            .arg("500")
            .arg("-y")
            .arg("1025")
            .arg("-z")
            .arg("120")
            .arg("-w")
            .arg("Kaiser")
            .arg("-S")
            .arg(&start_time)
            .arg("-d")
            .arg(&duration)
            .arg("-t")
            .arg(&self.image_title)
            .arg("-c")
            .arg("caesura")
            .arg("-o")
            .arg(&self.output_path)
            .run()
            .await
            .map_err(Failure::wrap_with_path(
                SpectrogramAction::GenerateSpectrogram,
                &self.output_path,
            ))
    }

    async fn execute_full(&self) -> Result<Output, Failure<SpectrogramAction>> {
        Command::new(SOX)
            .arg(&self.source_path)
            .arg("-n")
            .arg("remix")
            .arg("1")
            .arg("spectrogram")
            .arg("-x")
            .arg("3000")
            .arg("-y")
            .arg("513")
            .arg("-z")
            .arg("120")
            .arg("-w")
            .arg("Kaiser")
            .arg("-t")
            .arg(&self.image_title)
            .arg("-c")
            .arg("caesura")
            .arg("-o")
            .arg(&self.output_path)
            .run()
            .await
            .map_err(Failure::wrap_with_path(
                SpectrogramAction::GenerateSpectrogram,
                &self.output_path,
            ))
    }
}

/// Calculate the start time for zoom spectrogram.
///
/// For tracks >= [`TYPICAL_ZOOM_START`] + [`ZOOM_DURATION`], uses the standard position (1:00).
/// For shorter tracks, uses 50% of the duration minus half the capture window to center it.
#[expect(clippy::integer_division, reason = "sub-second precision not needed")]
fn calculate_zoom_start(duration_secs: Option<u32>) -> String {
    match duration_secs {
        Some(duration) if duration < TYPICAL_ZOOM_START + ZOOM_DURATION => {
            // Center the capture window at 50% of the track
            let midpoint = duration / 2;
            let start = midpoint.saturating_sub(ZOOM_DURATION / 2);
            format!("0:{start:02}")
        }
        _ => format!("{}:{:02}", TYPICAL_ZOOM_START / 60, TYPICAL_ZOOM_START % 60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_start_unknown_duration_uses_typical() {
        assert_eq!(calculate_zoom_start(None), "1:00");
    }

    #[test]
    fn zoom_start_longer_than_typical_uses_typical() {
        assert_eq!(calculate_zoom_start(Some(65)), "1:00");
        assert_eq!(calculate_zoom_start(Some(120)), "1:00");
        assert_eq!(calculate_zoom_start(Some(300)), "1:00");
    }

    #[test]
    fn zoom_start_at_threshold_uses_typical() {
        // Exactly at threshold (60 + 2 = 62 seconds)
        assert_eq!(calculate_zoom_start(Some(62)), "1:00");
    }

    #[test]
    fn zoom_start_shorter_than_typical_uses_midpoint() {
        // 30 second track: midpoint=15, start=15-1=14
        assert_eq!(calculate_zoom_start(Some(30)), "0:14");
        // 61 second track (just under threshold): midpoint=30, start=30-1=29
        assert_eq!(calculate_zoom_start(Some(61)), "0:29");
    }

    #[test]
    fn zoom_start_shorter_than_zoom_duration() {
        // 1 second track: midpoint=0, start=0-1=0 (saturating)
        assert_eq!(calculate_zoom_start(Some(1)), "0:00");
        // 2 second track: midpoint=1, start=1-1=0
        assert_eq!(calculate_zoom_start(Some(2)), "0:00");
        // 4 second track: midpoint=2, start=2-1=1
        assert_eq!(calculate_zoom_start(Some(4)), "0:01");
    }

    #[test]
    fn zoom_start_zero_duration() {
        assert_eq!(calculate_zoom_start(Some(0)), "0:00");
    }
}
