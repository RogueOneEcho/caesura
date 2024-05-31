use crate::source::SourceError;
use crate::source::SourceError::*;
use claxon::metadata::StreamInfo;

#[must_use]
pub fn is_resample_required(info: &StreamInfo) -> bool {
    info.sample_rate > 48000 || info.bits_per_sample > 16
}

pub fn get_resample_rate(info: &StreamInfo) -> Result<u32, SourceError> {
    if info.sample_rate % 44100 == 0 {
        Ok(44100)
    } else if info.sample_rate % 48000 == 0 {
        Ok(48000)
    } else {
        Err(UnknownSampleRate(info.sample_rate))
    }
}

pub fn validate(info: StreamInfo) -> Result<(), SourceError> {
    // TODO MUST perform channel count validation from SourceVerifier
    if info.channels > 2 {
        Err(TooManyChannels(info.channels))
    } else {
        Ok(())
    }
}
