use di::{injectable, Ref};

use crate::errors::claxon_error;
use crate::formats::target_format::TargetFormat;
use crate::fs::{convert_to_id3v2, fix_track_numbering, get_vorbis_tags, FlacFile, PathManager};
use crate::jobs::Job;
use crate::source::Source;
use crate::transcode::transcode_job::TranscodeJob;
use crate::transcode::*;
use rogue_logging::Error;

/// Create a [`TranscodeJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
#[injectable]
pub struct TranscodeJobFactory {
    paths: Ref<PathManager>,
}

impl TranscodeJobFactory {
    /// Create a [`TranscodeJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
    pub fn create(
        &self,
        flacs: &[FlacFile],
        source: &Source,
        format: TargetFormat,
    ) -> Result<Vec<Job>, Error> {
        let mut jobs = Vec::new();
        for (index, flac) in flacs.iter().enumerate() {
            jobs.push(self.create_single(index, flac, source, format)?);
        }
        Ok(jobs)
    }

    /// Create a single [`TranscodeJob`] from a `flac_file`.
    pub fn create_single(
        &self,
        index: usize,
        flac: &FlacFile,
        source: &Source,
        format: TargetFormat,
    ) -> Result<Job, Error> {
        let info = flac
            .get_stream_info()
            .map_err(|e| claxon_error(e, "read FLAC"))?;
        let id = format!("Transcode {:<4}{index:>3}", format.to_string());
        let output_path = self.paths.get_transcode_path(source, format, flac);
        let variant = if matches!(format, TargetFormat::Flac) && is_resample_required(&info) {
            Variant::Resample(Resample {
                input: flac.path.clone(),
                output: output_path.clone(),
                resample_rate: get_resample_rate(&info)?,
            })
        } else {
            let resample_rate = is_resample_required(&info)
                .then(|| get_resample_rate(&info))
                .transpose()?;
            Variant::Transcode(
                Decode {
                    input: flac.path.clone(),
                    resample_rate,
                },
                Encode {
                    format,
                    output: output_path.clone(),
                },
            )
        };
        let tags = if matches!(format, TargetFormat::_320 | TargetFormat::V0) {
            let mut tags = get_vorbis_tags(flac)?;
            convert_to_id3v2(&mut tags);
            let _ = fix_track_numbering(&mut tags);
            Some(tags)
        } else {
            None
        };
        Ok(Job::Transcode(TranscodeJob { id, variant, tags }))
    }
}
