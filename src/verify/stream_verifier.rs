use crate::errors::AppError;
use crate::fs::FlacFile;
use crate::transcode::get_resample_rate;
use crate::verify::SourceRule;
use crate::verify::SourceRule::{TooManyChannels, UnknownSampleRate};

pub struct StreamVerifier;

impl StreamVerifier {
    pub fn execute(flac: &FlacFile) -> Result<Vec<SourceRule>, AppError> {
        let mut errors = Vec::new();
        let info = flac.get_stream_info()?;
        if get_resample_rate(&info).is_none() {
            errors.push(UnknownSampleRate(info.sample_rate));
        }
        if info.channels > 2 {
            errors.push(TooManyChannels(info.channels));
        }
        Ok(errors)
    }
}
