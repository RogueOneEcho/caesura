use crate::prelude::*;

/// Job to resize and copy an additional file to the transcode directory.
pub(crate) struct AdditionalJob {
    /// Job identifier for progress tracking.
    pub id: String,
    /// Resize operation to perform.
    pub resize: Resize,
}

impl AdditionalJob {
    #[allow(clippy::integer_division)]
    pub(crate) async fn execute(self) -> Result<(), Failure<TranscodeAction>> {
        trace!(
            "{} image to maximum {} px and {}% quality: {}",
            "Resizing".bold(),
            self.resize.max_pixel_size,
            self.resize.quality,
            self.resize.input.display()
        );
        let output = self.resize.output.clone();
        let info = self.resize.to_info();
        trace!("{info}");
        info.to_command()
            .run()
            .await
            .map_err(Failure::wrap_with_path(
                TranscodeAction::ResizeImage,
                &output,
            ))?;
        Ok(())
    }
}
