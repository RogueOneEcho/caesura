use crate::commands::*;
use crate::options::*;
use crate::utils::*;
use colored::Colorize;
use di::{Ref, injectable};
use log::{trace, warn};
use rogue_logging::Error;
use tokio::fs::{copy, create_dir_all, hard_link};

pub const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];
pub const TEXT_EXTENSIONS: [&str; 5] = ["cue", "log", "nfo", "rtf", "txt"];

/// Factory for creating [`AdditionalJob`] instances.
#[injectable]
pub(crate) struct AdditionalJobFactory {
    copy_options: Ref<CopyOptions>,
    file_options: Ref<FileOptions>,
    paths: Ref<PathManager>,
}

impl AdditionalJobFactory {
    /// Create an [`AdditionalJob`] for each [`AdditionalFile`].
    pub(crate) async fn create(
        &self,
        files: &[AdditionalFile],
        source: &Source,
        target: TargetFormat,
    ) -> Result<Vec<Job>, Error> {
        let mut jobs = Vec::new();
        for (index, file) in files.iter().enumerate() {
            if let Some(job) = self.create_single(index, file, source, target).await? {
                jobs.push(job);
            }
        }
        Ok(jobs)
    }

    /// Create a single [`AdditionalJob`] from a `flac_file`.
    #[allow(clippy::integer_division)]
    async fn create_single(
        &self,
        index: usize,
        file: &AdditionalFile,
        source: &Source,
        target: TargetFormat,
    ) -> Result<Option<Job>, Error> {
        let source_path = file.path.clone();
        let output_dir = self
            .paths
            .get_transcode_target_dir(source, target)
            .join(&file.sub_dir);
        let mut output_path = output_dir.join(&file.file_name);
        let size = file.get_size().await?;
        let max_file_size = self
            .file_options
            .max_file_size
            .expect("max_file_size should be set");
        let extension = source_path
            .extension()
            .expect("Source has extension")
            .to_string_lossy()
            .to_string()
            .to_lowercase();
        let is_image = IMAGE_EXTENSIONS.contains(&extension.as_str());
        let is_large = size > max_file_size;
        let no_image_compression = self
            .file_options
            .no_image_compression
            .expect("no_image_compression should be set");
        create_dir_all(&output_dir)
            .await
            .map_err(|e| io_error(e, "create directories for additional file"))?;
        if !is_image || no_image_compression || !is_large {
            if is_large {
                warn!(
                    "Including large {} ({} KB): {}",
                    extension,
                    size / 1_000,
                    source_path.display()
                );
            }
            let hard_link_option = self
                .copy_options
                .hard_link
                .expect("hard_link should be set");
            let verb = if hard_link_option {
                hard_link(&source_path, &output_path)
                    .await
                    .map_err(|e| io_error(e, "hard link additional file"))?;
                "Hard Linked"
            } else {
                copy(&source_path, &output_path)
                    .await
                    .map_err(|e| io_error(e, "copy additional file"))?;
                "Copied"
            };
            trace!(
                "{} {} to {}",
                verb.bold(),
                &source_path.display(),
                &output_path.display()
            );
            return Ok(None);
        }
        let no_png_to_jpg = self
            .file_options
            .no_png_to_jpg
            .expect("no_png_to_jpg should be set");
        if !no_png_to_jpg && extension == "png" {
            let mut temp_source = source_path.clone();
            temp_source.set_extension("jpg");
            if temp_source.exists() {
                output_path.set_extension("png.jpg");
            } else {
                output_path.set_extension("jpg");
            }
        }
        let id = format!("Additional {target:<7?}{index:>3}");
        let max_pixel_size = self
            .file_options
            .max_pixel_size
            .expect("max_pixel_size should be set");
        let quality = self
            .file_options
            .jpg_quality
            .expect("jpg_quality should be set");
        let job = Job::Additional(AdditionalJob {
            id,
            resize: Resize {
                input: source_path,
                output: output_path,
                max_pixel_size,
                quality,
            },
        });
        Ok(Some(job))
    }
}
