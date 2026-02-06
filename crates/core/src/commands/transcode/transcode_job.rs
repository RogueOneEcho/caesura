use crate::prelude::*;
use lofty::config::WriteOptions;
use lofty::prelude::TagExt;
use lofty::tag::ItemKey::{Popularimeter, Work};
use lofty::tag::Tag;
use std::fs::create_dir_all;
use std::process::Stdio;
use tokio::fs::{copy, hard_link};
use tokio::join;

/// Job to transcode a single FLAC file to a target format.
pub(crate) struct TranscodeJob {
    /// Job identifier for progress tracking.
    pub id: String,
    /// Transcode operation to perform.
    pub variant: Variant,
    /// ID3 tags to write to MP3 output.
    pub tags: Option<Tag>,
}

impl TranscodeJob {
    pub(crate) async fn execute(self) -> Result<(), Failure<TranscodeAction>> {
        let output_path = match &self.variant {
            Variant::Transcode(_, encode) => encode.output.clone(),
            Variant::Resample(resample) => resample.output.clone(),
            Variant::Include(include) => include.output.clone(),
        };
        let output_dir = output_path
            .parent()
            .expect("output path should have a parent");
        create_dir_all(output_dir).map_err(Failure::wrap_with_path(
            TranscodeAction::CreateOutputDirectory,
            output_dir,
        ))?;
        match self.variant {
            Variant::Transcode(decode, encode) => execute_transcode(decode, encode).await?,
            Variant::Resample(resample) => execute_resample(resample).await?,
            Variant::Include(include) => execute_include(include).await?,
        }
        if let Some(mut tags) = self.tags {
            let exclude = [Popularimeter, Work];
            for key in exclude {
                if let Some(value) = tags.get_string(&key) {
                    trace!("Excluding invalid {key:?} value: {value}");
                    tags.remove_key(&key);
                }
            }
            tags.save_to_path(&output_path, WriteOptions::default())
                .map_err(Failure::wrap_with_path(
                    TranscodeAction::WriteTags,
                    &output_path,
                ))?;
        }
        Ok(())
    }
}

/// Pipe decode output directly to encode input.
async fn execute_transcode(decode: Decode, encode: Encode) -> Result<(), Failure<TranscodeAction>> {
    let decode_info = decode.to_info();
    let encode_info = encode.to_info();
    trace!("Executing transcode: {decode_info} | {encode_info}");
    let decode_program = decode_info.program.clone();
    let mut decode_command = decode_info
        .to_command()
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Failure::wrap_with(TranscodeAction::SpawnDecode, |f| {
            f.with("program", &decode_program)
        }))?;
    let pipe: Stdio = decode_command
        .stdout
        .take()
        .expect("should be able to take stdout")
        .try_into()
        .expect("should be able to convert stdout to pipe");
    let encode_program = encode_info.program.clone();
    let encode_command = encode_info
        .to_command()
        .stdin(pipe)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Failure::wrap_with(TranscodeAction::SpawnEncode, |f| {
            f.with("program", &encode_program)
        }))?;
    let (decode_result, encode_output) =
        join!(decode_command.wait(), encode_command.wait_with_output());
    let decode_exit = decode_result.map_err(Failure::wrap(TranscodeAction::WaitDecode))?;
    let encode_output = encode_output.map_err(Failure::wrap(TranscodeAction::WaitEncode))?;
    if !decode_exit.success() {
        warn!("Decode ({decode_program}) was not successful: {decode_exit}");
    }
    require_success(encode_output, &encode_program).map_err(Failure::wrap_with_path(
        TranscodeAction::Transcode,
        encode_program,
    ))?;
    Ok(())
}

async fn execute_resample(resample: Resample) -> Result<(), Failure<TranscodeAction>> {
    let output = resample.output.clone();
    let info = resample.to_info();
    trace!("Executing resample: {info}");
    info.to_command()
        .run()
        .await
        .map_err(Failure::wrap_with_path(TranscodeAction::Resample, &output))?;
    Ok(())
}

async fn execute_include(include: Include) -> Result<(), Failure<TranscodeAction>> {
    let verb = if include.hard_link {
        hard_link(&include.input, &include.output)
            .await
            .map_err(Failure::wrap_with_path(
                TranscodeAction::HardLinkFlac,
                &include.output,
            ))?;
        "Hard Linked"
    } else {
        copy(&include.input, &include.output)
            .await
            .map_err(Failure::wrap_with_path(
                TranscodeAction::CopyFlac,
                &include.output,
            ))?;
        "Copied"
    };
    trace!(
        "{} {} to {}",
        verb.bold(),
        &include.input.display(),
        &include.output.display()
    );
    Ok(())
}
