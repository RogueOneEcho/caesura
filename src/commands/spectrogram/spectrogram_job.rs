use rogue_logging::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Output;
use tokio::process::Command;

use crate::commands::*;
use crate::dependencies::*;
use crate::utils::*;

/// A command to generate a spectrogram image of a FLAC file using sox.
///
/// A [command design pattern](https://refactoring.guru/design-patterns/command) is used
/// so the execution of the command can be deferred and multiple commands can be executed
/// in parallel via the multithreaded [`SpectrogramCommandRunner`].
pub(crate) struct SpectrogramJob {
    pub id: String,
    pub source_path: String,
    pub output_path: PathBuf,
    pub image_title: String,
    pub size: Size,
}

impl SpectrogramJob {
    /// Execute the command to generate the spectrogram.
    pub(crate) async fn execute(self) -> Result<(), Error> {
        let output_dir = self
            .output_path
            .parent()
            .expect("output path should have a parent");
        create_dir_all(output_dir)
            .map_err(|e| io_error(e, "create spectrogram output directory"))?;
        match self.size {
            Size::Full => self.execute_full().await,
            Size::Zoom => self.execute_zoom().await,
        }?;
        Ok(())
    }

    async fn execute_zoom(&self) -> Result<Output, Error> {
        let output = Command::new(SOX)
            .arg(&self.source_path)
            .arg("-n")
            .arg("remix")
            .arg("1")
            .arg("spectrogram")
            .arg("-x")
            .arg("500")
            .arg("-y")
            .arg("1025")
            .arg("-z")
            .arg("120")
            .arg("-w")
            .arg("Kaiser")
            .arg("-S")
            .arg("1:00")
            .arg("-d")
            .arg("0:02")
            .arg("-t")
            .arg(&self.image_title)
            .arg("-c")
            .arg("red_oxide")
            .arg("-o")
            .arg(&self.output_path)
            .output()
            .await
            .map_err(|e| command_error(e, "execute generate spectrogram", SOX))?;
        OutputHandler::execute(output, "generate spectrogram", "IMDL")
    }

    async fn execute_full(&self) -> Result<Output, Error> {
        let output = Command::new(SOX)
            .arg(&self.source_path)
            .arg("-n")
            .arg("remix")
            .arg("1")
            .arg("spectrogram")
            .arg("-x")
            .arg("3000")
            .arg("-y")
            .arg("513")
            .arg("-z")
            .arg("120")
            .arg("-w")
            .arg("Kaiser")
            .arg("-t")
            .arg(&self.image_title)
            .arg("-c")
            .arg("red_oxide")
            .arg("-o")
            .arg(&self.output_path)
            .output()
            .await
            .map_err(|e| command_error(e, "execute generate spectrogram", SOX))?;
        OutputHandler::execute(output, "generate spectrogram", "IMDL")
    }
}
