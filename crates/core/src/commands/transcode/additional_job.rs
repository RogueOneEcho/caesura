use tokio::task::spawn_blocking;

use crate::prelude::*;

/// Job to resize and copy an additional file to the transcode directory.
pub(crate) struct AdditionalJob {
    /// Job identifier for progress tracking.
    pub id: String,
    /// Resize operation to perform.
    pub resize: Resize,
}

impl AdditionalJob {
    pub(crate) async fn execute(self) -> Result<(), Failure<TranscodeAction>> {
        trace!(
            "{} image to maximum {} px and {}% quality: {}",
            "Resizing".bold(),
            self.resize.max_pixel_size,
            self.resize.quality,
            self.resize.input.display()
        );
        let output = self.resize.output.clone();
        spawn_blocking(move || self.resize.execute())
            .await
            .expect("resize task should not panic")
            .map_err(Failure::wrap_with_path(
                TranscodeAction::ResizeImage,
                &output,
            ))?;
        Ok(())
    }
}
