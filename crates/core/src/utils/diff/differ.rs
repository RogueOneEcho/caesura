use crate::prelude::*;
use similar::TextDiff;

/// Diff with visualization of non-UTF-8 values.
///
/// - Non-UTF-8 values are converted to hex: `<A1>`
/// - Uses grapheme aware diff logic
/// - Output rendered with [`DiffRenderer`]
pub(crate) struct Differ {
    renderer: DiffRenderer,
}

impl Differ {
    pub fn new(renderer: DiffRenderer) -> Self {
        Self { renderer }
    }

    /// Diff `old` against `new`.
    ///
    ///
    pub fn execute(&self, old: &RawString, new: &str) -> String {
        let old = old.to_string_with_hex();
        let diff = TextDiff::from_graphemes(old.as_str(), new);
        let diff = merge_consecutive(&diff);
        self.renderer.render(diff)
    }
}

/// Merge consecutive same-tag changes of `diff` into [`DiffSpan`].
fn merge_consecutive(diff: &TextDiff<'_, '_, str>) -> Vec<DiffSpan> {
    let mut spans: Vec<DiffSpan> = Vec::new();
    for change in diff.iter_all_changes() {
        match spans.last_mut() {
            Some(span) if span.tag == change.tag() => span.value.push_str(change.value()),
            _ => spans.push(DiffSpan {
                tag: change.tag(),
                value: change.value().to_owned(),
            }),
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differ_execute_colored() {
        // Arrange
        let old = RawString::Bytes(splice(b"song", E_ACUTE, b".flac"));
        let new = "songé.flac";
        let renderer = DiffRenderer::colored();
        let differ = Differ::new(renderer);

        // Act
        let output = differ.execute(&old, new);

        // Assert
        assert_eq!(
            output,
            format!(
                "{}{}{}{}",
                "song".dimmed(),
                "<E9>".red(),
                "é".green(),
                ".flac".dimmed()
            )
        );
    }

    #[test]
    fn differ_execute_bb() {
        // Arrange
        let old = RawString::Bytes(splice(b"song", E_ACUTE, b".flac"));
        let new = "songé.flac";
        let renderer = DiffRenderer::colored_bb_code();
        let differ = Differ::new(renderer);

        // Act
        let output = differ.execute(&old, new);

        // Assert
        assert_eq!(
            output,
            format!(
                "{}{}{}{}",
                "song".dimmed(),
                "[color=red]<E9>[/color]".red(),
                "[color=green]é[/color]".green(),
                ".flac".dimmed(),
            )
        );
    }

    #[test]
    fn differ_execute_bb_libtorrent() {
        // Arrange
        let old = RawString::Bytes(splice(b"song", E_ACUTE, b".flac"));
        let new = "song_lac";
        let renderer = DiffRenderer::colored_bb_code();
        let differ = Differ::new(renderer);

        // Act
        let output = differ.execute(&old, new);

        // Assert
        assert_eq!(
            output,
            format!(
                "{}{}{}{}",
                "song".dimmed(),
                "[color=red]<E9>.f[/color]".red(),
                "[color=green]_[/color]".green(),
                "lac".dimmed()
            )
        );
    }
}
