use crate::prelude::*;
use lofty::prelude::Accessor;

/// Default max filename length (excluding extension).
const DEFAULT_MAX_FILESTEM_LENGTH: usize = 70;

impl FlacFile {
    /// Get renamed track file stem (without extension) using the disc context.
    ///
    /// Falls back to original file stem if context not set or tags are missing.
    #[must_use]
    pub fn renamed_file_stem(&self) -> String {
        self.disc_context
            .as_ref()
            .and_then(|ctx| format_track_name(self, ctx.track_padding, None))
            .unwrap_or_else(|| self.file_name.clone())
    }

    /// Get subdirectory for track using the disc context.
    ///
    /// Returns `Some(CD{N}/)` for multi-disc releases, `None` otherwise.
    #[must_use]
    pub fn renamed_sub_dir(&self) -> Option<PathBuf> {
        let ctx = self.disc_context.as_ref()?;
        if !ctx.is_multi_disc {
            return None;
        }
        let disc = self.disc_number()?;
        Some(PathBuf::from(format!("CD{disc}")))
    }

    /// Get disc number from ID3 tags.
    #[must_use]
    fn disc_number(&self) -> Option<u32> {
        let tags = self.id3_tags().ok()?;
        tags.disk()
    }
}

/// Get track filename with specified padding width.
///
/// Truncates title if resulting name exceeds `max_length` (defaults to `DEFAULT_MAX_FILENAME_LENGTH`).
#[must_use]
fn format_track_name(flac: &FlacFile, padding: usize, max_length: Option<usize>) -> Option<String> {
    let tags = flac.id3_tags().ok()?;
    let track_number = tags.track()?;
    let title = tags.title()?;
    let sanitized_title = Sanitizer::execute(title.to_string());
    let max_len = max_length.unwrap_or(DEFAULT_MAX_FILESTEM_LENGTH);
    let max_title_len = max_len.saturating_sub(padding + 1);
    let truncated_title: String = if sanitized_title.chars().count() > max_title_len {
        sanitized_title.chars().take(max_title_len).collect()
    } else {
        sanitized_title
    };
    Some(format!("{track_number:0>padding$} {truncated_title}"))
}
