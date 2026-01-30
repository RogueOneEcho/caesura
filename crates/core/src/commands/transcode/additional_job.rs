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
    pub(crate) async fn execute(self) -> Result<(), Error> {
        trace!(
            "{} image to maximum {} px and {}% quality: {}",
            "Resizing".bold(),
            self.resize.max_pixel_size,
            self.resize.quality,
            self.resize.input.display()
        );
        let info = self.resize.to_info();
        trace!("{info}");
        let output = info
            .to_command()
            .output()
            .await
            .map_err(|e| command_error(e, "execute resize image", CONVERT))?;
        OutputHandler::execute(output, "resize image", "convert")?;
        Ok(())
    }
}
