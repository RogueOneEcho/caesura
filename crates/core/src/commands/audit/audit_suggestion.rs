use crate::prelude::*;

/// A candidate decoding of a non-UTF-8 path element.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AuditSuggestion {
    /// Where the encoding came from.
    pub kind: AuditSuggestionKind,
    /// Encoding label used to decode, e.g. `windows-1252` or `Shift_JIS`.
    pub encoding: String,
    /// The element decoded to UTF-8.
    pub value: String,
}

/// Origin of a [`AuditSuggestion`] encoding.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum AuditSuggestionKind {
    /// The torrent's declared top-level `encoding` field.
    EncodingField,
    /// Statistical detection by `chardetng`.
    Chardetng,
    /// Fallback assumption of Windows-1252.
    Windows1252,
    /// Surrogate-pair decoding of CESU-8 bytes.
    ///
    /// Recovers astral characters, such as emoji, that a tool encoded as
    /// UTF-16 surrogate pairs rather than as 4-byte UTF-8.
    Cesu8,
}
