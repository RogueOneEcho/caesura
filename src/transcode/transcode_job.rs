use std::fs::create_dir_all;
use std::process::Stdio;

use audiotags::{AudioTagWrite, Id3v2Tag};
use colored::Colorize;
use log::*;
use tokio::io::AsyncWriteExt;

use crate::errors::{AppError, OutputHandler};
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
        let action = "transcode";
        create_dir_all(&self.output_dir).or_else(|e| AppError::io(e, action))?;

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
            let output = OutputHandler::execute(output, action, "transcode")?;
            buffer = output.stdout;
        }
        if let Some(tags) = self.tags {
            let mut tags = tags;
            tags.write_to_path(self.output_path.as_str())
                .or_else(|e| AppError::tag(e, "write tags to file"))?;
        }
        Ok(())
    }
}
