use crate::commands::*;
use crate::utils::*;

use lofty::config::WriteOptions;
use lofty::prelude::TagExt;
use lofty::tag::ItemKey::{Popularimeter, Work};
use lofty::tag::Tag;
use log::{trace, warn};
use rogue_logging::Error;
use std::fs::create_dir_all;
use std::process::Stdio;
use tokio::join;

pub(crate) struct TranscodeJob {
    pub id: String,
    pub variant: Variant,
    pub tags: Option<Tag>,
}

impl TranscodeJob {
    pub(crate) async fn execute(self) -> Result<(), Error> {
        let output_path = match &self.variant {
            Variant::Transcode(_, encode) => encode.output.clone(),
            Variant::Resample(resample) => resample.output.clone(),
        };
        let output_dir = output_path
            .parent()
            .expect("output path should have a parent");
        create_dir_all(output_dir).map_err(|e| io_error(e, "create transcode output directory"))?;
        match self.variant {
            Variant::Transcode(decode, encode) => execute_transcode(decode, encode).await?,
            Variant::Resample(resample) => execute_resample(resample).await?,
        };
        if let Some(mut tags) = self.tags {
            let exclude = [Popularimeter, Work];
            for key in exclude {
                if let Some(value) = tags.get_string(&key) {
                    trace!("Excluding invalid {key:?} value: {value}");
                    tags.remove_key(&key);
                }
            }
            tags.save_to_path(&output_path, WriteOptions::default())
                .map_err(|e| error("write tags", e.to_string()))?;
        }
        Ok(())
    }
}

async fn execute_transcode(decode: Decode, encode: Encode) -> Result<(), Error> {
    let decode_info = decode.to_info();
    let encode_info = encode.to_info();
    trace!("Executing transcode: {decode_info} | {encode_info}");
    let decode_program = decode_info.program.clone();
    let mut decode_command = decode_info
        .to_command()
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| command_error(e, "spawn decode", &decode_program))?;
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
        .map_err(|e| command_error(e, "spawn decode", &encode_program))?;
    let (decode_result, encode_output) =
        join!(decode_command.wait(), encode_command.wait_with_output());
    let decode_exit = decode_result.map_err(|e| io_error(e, "wait for decode"))?;
    let encode_output = encode_output.map_err(|e| io_error(e, "wait for encode"))?;
    if !decode_exit.success() {
        warn!("Decode was not successful: {decode_exit}");
    }
    OutputHandler::execute(encode_output, "execute resample job", "transcode")?;
    Ok(())
}

async fn execute_resample(resample: Resample) -> Result<(), Error> {
    let info = resample.to_info();
    trace!("Executing resample: {info}");
    let program = info.program.clone();
    let output = info
        .to_command()
        .output()
        .await
        .map_err(|e| command_error(e, "execute resample job", &program))?;
    OutputHandler::execute(output, "execute resample job", "transcode")?;
    Ok(())
}
