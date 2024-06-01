use crate::errors::AppError;
use claxon::metadata::StreamInfo;

#[must_use]
pub fn is_resample_required(info: &StreamInfo) -> bool {
    info.sample_rate > 48000 || info.bits_per_sample > 16
}

#[must_use]
pub fn get_resample_rate(info: &StreamInfo) -> Option<u32> {
    if info.sample_rate % 44100 == 0 {
        Some(44100)
    } else if info.sample_rate % 48000 == 0 {
        Some(48000)
    } else {
        None
    }
}

pub fn get_resample_rate_or_err(info: &StreamInfo) -> Result<u32, AppError> {
    get_resample_rate(info).ok_or_else(|| {
        AppError::else_explained("get sample rate", "invalid sample rate".to_owned())
    })
}
