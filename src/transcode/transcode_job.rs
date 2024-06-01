use std::process::Stdio;

use crate::errors::AppError;
use crate::jobs::AppError::{IOFailure, SourceFailure, TranscodeFailure};
use audiotags::{AudioTagWrite, Id3v2Tag};
use colored::Colorize;
use log::*;
use tokio::io::AsyncWriteExt;

use crate::logging::Colors;
use crate::source::SourceError::AudioTagFailure;
use crate::transcode::CommandFactory;

pub struct TranscodeJob {
    pub id: String,
    pub output_dir: String,
    pub output_path: String,
    pub commands: Vec<CommandFactory>,
    pub tags: Option<Id3v2Tag>,
}

impl TranscodeJob {
    pub async fn execute(self) -> Result<(), AppError> {
        if let Err(error) = std::fs::create_dir_all(&self.output_dir) {
            return Err(IOFailure(error));
        }
        let mut buffer = vec![];
        for factory in self.commands {
            let command = format!(
                "{} \"{}\"",
                factory.program.clone(),
                factory.args.clone().join("\" \"")
            );
            trace!("{} {}", "Executing".bold(), command);
            let mut child = factory
                .create()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                // TODO SHOULD do something with stderr or use Stdio::null()
                .stderr(Stdio::piped())
                .spawn()
                .expect("Process should be able to spawn");
            if !buffer.is_empty() {
                let mut stdin = child.stdin.take().expect("stdin should be available");
                stdin
                    .write_all(&buffer)
                    .await
                    .expect("Should be able to write to std in");
                drop(stdin);
            }
            let output = child
                .wait_with_output()
                .await
                .expect("Child should produce an output");
            if output.status.success() {
                buffer = output.stdout;
            } else {
                error!("{} to execute {}", "Failed".red(), command.gray());
                debug!("{} {:?}", "Exit code:".bold(), output.status.code());
                let out = String::from_utf8(output.stdout).unwrap_or_default();
                debug!("{}\n{}", "Out:".bold(), out);
                let err = String::from_utf8(output.stderr).unwrap_or_default();
                debug!("{}\n{}", "Err:".bold(), err);
                return Err(TranscodeFailure);
            }
        }
        if let Some(tags) = self.tags {
            let mut tags = tags;
            if let Err(error) = tags.write_to_path(self.output_path.as_str()) {
                return Err(SourceFailure(AudioTagFailure(error)));
            }
        }
        Ok(())
    }
}
