use std::path::Path;

use crate::errors::AppError;
use audiotags::Id3v2Tag;
use di::injectable;

use crate::formats::target_format::TargetFormat;
use crate::fs::FlacFile;
use crate::jobs::Job;
use crate::transcode::transcode_job::TranscodeJob;
use crate::transcode::*;

#[injectable]
pub struct TranscodeJobFactory;

impl TranscodeJobFactory {
    /// Create a [`TranscodeJob`] for each [`FlacFile`] in the [`Vec<FlacFile>`].
    pub fn create(
        &self,
        flacs: &[FlacFile],
        format: TargetFormat,
        output_dir: &Path,
    ) -> Result<Vec<Job>, AppError> {
        let mut jobs = Vec::new();
        for (index, flac) in flacs.iter().enumerate() {
            jobs.push(self.create_single(index, flac, format, output_dir)?);
        }
        Ok(jobs)
    }

    /// Create a single [`TranscodeJob`] from a `flac_file`.
    fn create_single(
        &self,
        index: usize,
        flac: &FlacFile,
        format: TargetFormat,
        output_dir: &Path,
    ) -> Result<Job, AppError> {
        let info = flac.get_stream_info()?;
        let id = format!("Transcode {format:<7?}{index:>3}");
        let output_dir = output_dir.join(&flac.sub_dir);
        let output_path = get_output_path(flac, format, &output_dir);
        let commands = if matches!(format, TargetFormat::Flac) && is_resample_required(&info) {
            let cmd = CommandFactory::new_flac_resample(flac, &info, output_path.clone())?;
            vec![cmd]
        } else {
            let decode_cmd = CommandFactory::new_decode(flac, &info)?;
            let encode_cmd = CommandFactory::new_encode(format, output_path.clone());
            vec![decode_cmd, encode_cmd]
        };
        let output_dir = output_dir.to_string_lossy().into_owned();
        let tags = if matches!(format, TargetFormat::_320) || matches!(format, TargetFormat::V0) {
            let tags = flac.get_tags()?;
            Some(Id3v2Tag::from(tags))
        } else {
            None
        };
        Ok(Job::Transcode(TranscodeJob {
            id,
            output_dir,
            output_path,
            commands,
            tags,
        }))
    }
}

fn get_output_path(flac: &FlacFile, format: TargetFormat, output_dir: &Path) -> String {
    let extension = format.get_file_extension();
    let filename = flac.file_name.clone() + "." + extension.as_str();
    output_dir.join(filename).to_string_lossy().into_owned()
}
