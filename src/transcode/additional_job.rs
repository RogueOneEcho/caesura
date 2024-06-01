use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Output;

use colored::Colorize;
use log::{trace, warn};
use tokio::fs::{copy, create_dir_all, hard_link, File};
use tokio::process::Command;

use crate::dependencies::CONVERT;
use crate::errors::{AppError, OutputHandler};

const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "gif"];
const MAX_FILE_SIZE: u64 = 750_000;
const MAX_PIXEL_SIZE: u32 = 1920_u32;
const QUALITY: u32 = 80_u32;

pub struct AdditionalJob {
    pub id: String,
    pub source_path: PathBuf,
    pub output_dir: String,
    pub output_path: String,
    pub hard_link: bool,
    pub compress_images: bool,
    pub extension: String,
}

impl AdditionalJob {
    #[allow(clippy::integer_division)]
    pub async fn execute(self) -> Result<(), AppError> {
        let file = File::open(&self.source_path)
            .await
            .or_else(|e| AppError::io(e, "open file"))?;
        let metadata = file
            .metadata()
            .await
            .or_else(|e| AppError::io(e, "read metadata"))?;
        let size = metadata.size();
        let is_large = size > MAX_FILE_SIZE;
        let is_image = IMAGE_EXTENSIONS.contains(&self.extension.as_str());
        if is_large && (!is_image || !self.compress_images) {
            warn!(
                "Including large {} ({} KB): {:?}",
                self.extension,
                size / 1_000,
                self.source_path
            );
        }
        create_dir_all(&self.output_dir)
            .await
            .or_else(|e| AppError::io(e, "create directories"))?;

        let verb = if is_large && is_image && self.compress_images {
            compress_image(
                &self.source_path.to_string_lossy().into_owned(),
                &self.output_path,
            )
            .await?;
            "Compressed"
        } else if self.hard_link {
            hard_link(&self.source_path, &self.output_path)
                .await
                .or_else(|e| AppError::io(e, "hard link file"))?;
            "Hard Linked"
        } else {
            copy(&self.source_path, &self.output_path)
                .await
                .or_else(|e| AppError::io(e, "copy file"))?;
            "Copied"
        };
        trace!(
            "{} {:?} to {}",
            verb.bold(),
            &self.source_path,
            &self.output_path
        );

        Ok(())
    }
}

async fn compress_image(source_path: &String, output_path: &String) -> Result<Output, AppError> {
    trace!(
        "{} image to maximum {} px and {}% quality: {}",
        "Reducing".bold(),
        MAX_PIXEL_SIZE,
        QUALITY,
        source_path
    );
    let output = Command::new(CONVERT)
        .arg(source_path)
        .arg("-resize")
        .arg(format!("{MAX_PIXEL_SIZE}x{MAX_PIXEL_SIZE}"))
        .arg("-quality")
        .arg(format!("{QUALITY}%"))
        .arg(output_path)
        .output()
        .await
        .or_else(|e| AppError::io(e, "compressing image"))?;
    OutputHandler::execute(output, "compressing image", "convert")
}
