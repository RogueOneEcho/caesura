use crate::commands::*;
use crate::utils::SourceIssue::*;
use crate::utils::*;

/// Verify FLAC stream properties are suitable for transcoding.
pub(crate) struct StreamVerifier;

impl StreamVerifier {
    pub(crate) fn execute(flac: &FlacFile) -> Vec<SourceIssue> {
        let mut errors = Vec::new();
        let info = match flac.get_stream_info() {
            Ok(info) => info,
            Err(claxon_error) => {
                errors.push(FlacError {
                    path: flac.path.clone(),
                    error: format!("{claxon_error}"),
                });
                return errors;
            }
        };
        if get_resample_rate(&info).is_err() {
            errors.push(SampleRate {
                path: flac.path.clone(),
                rate: info.sample_rate,
            });
        }
        if let Some(rate) = get_average_bit_rate(&info)
            && rate < MIN_BIT_RATE_KBPS * 1000
        {
            errors.push(BitRate {
                path: flac.path.clone(),
                rate,
            });
        }
        if let Some(seconds) = get_duration(&info)
            && seconds > MAX_DURATION
        {
            errors.push(Duration {
                path: flac.path.clone(),
                seconds,
            });
        }
        if info.channels > 2 {
            errors.push(Channels {
                path: flac.path.clone(),
                count: info.channels,
            });
        }
        errors
    }
}
