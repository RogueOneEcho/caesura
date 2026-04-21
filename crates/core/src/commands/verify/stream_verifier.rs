use crate::prelude::*;
use claxon::metadata::StreamInfo;

/// Verify FLAC stream properties are suitable for transcoding.
pub(crate) struct StreamVerifier;

impl StreamVerifier {
    /// Verify stream properties of a FLAC file and return any issues found.
    pub(crate) fn execute(flac: &FlacFile) -> Vec<SourceIssue> {
        let info = match check_flac_readable(flac) {
            Ok(info) => info,
            Err(issue) => return vec![issue],
        };
        let mut issues = Vec::new();
        issues.extend(check_sample_rate(&flac.path, &info));
        issues.extend(check_bit_rate(&flac.path, &info));
        issues.extend(check_duration(&flac.path, &info));
        issues.extend(check_channels(&flac.path, &info));
        issues
    }
}

/// Check the FLAC file can be read and return its stream info.
pub(crate) fn check_flac_readable(flac: &FlacFile) -> Result<StreamInfo, SourceIssue> {
    flac.get_stream_info().map_err(|e| SourceIssue::FlacError {
        path: flac.path.clone(),
        error: format!("{e}"),
    })
}

/// Check the sample rate is a multiple of 44100 or 48000.
pub(crate) fn check_sample_rate(path: &Path, info: &StreamInfo) -> Option<SourceIssue> {
    if get_resample_rate(info).is_err() {
        return Some(SourceIssue::SampleRate {
            path: path.to_path_buf(),
            rate: info.sample_rate,
        });
    }
    None
}

/// Check the average bit rate meets the minimum threshold.
pub(crate) fn check_bit_rate(path: &Path, info: &StreamInfo) -> Option<SourceIssue> {
    if let Some(rate) = get_average_bit_rate(info)
        && rate < MIN_BIT_RATE_KBPS * 1000
    {
        return Some(SourceIssue::BitRate {
            path: path.to_path_buf(),
            rate,
        });
    }
    None
}

/// Check the duration does not exceed the maximum.
pub(crate) fn check_duration(path: &Path, info: &StreamInfo) -> Option<SourceIssue> {
    if let Some(seconds) = get_duration(info)
        && seconds > MAX_DURATION
    {
        return Some(SourceIssue::Duration {
            path: path.to_path_buf(),
            seconds,
        });
    }
    None
}

/// Check the channel count does not exceed 2.
pub(crate) fn check_channels(path: &Path, info: &StreamInfo) -> Option<SourceIssue> {
    if info.channels > 2 {
        return Some(SourceIssue::Channels {
            path: path.to_path_buf(),
            count: info.channels,
        });
    }
    None
}
