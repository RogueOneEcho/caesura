use similar::ChangeTag;

/// A maximal span of consecutive graphemes sharing one [`ChangeTag`].
pub(crate) struct DiffSpan {
    /// The change classification shared by every grapheme in the span.
    pub tag: ChangeTag,
    /// The concatenated grapheme values.
    pub value: String,
}
