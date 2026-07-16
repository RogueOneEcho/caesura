use crate::prelude::*;

/// A problem found in a torrent file path.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditIssue {
    /// Category of the problem found.
    pub kind: AuditIssueKind,
    /// Source bytes of the offending path component.
    pub raw: Option<RawString>,
    /// Candidate decodings of the raw bytes, best-first.
    pub suggestions: Option<Vec<AuditSuggestion>>,
    /// Sanitized characters found.
    pub sanitized: Option<HashSet<SanitizerChar>>,
}

/// Category of a problem found while auditing a torrent.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, ThisError)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum AuditIssueKind {
    #[default]
    #[error("unknown")]
    Unknown,
    #[error("parse torrent")]
    Parse,
    #[error("no root element")]
    NoRootElement,
    /// The root element is not a dictionary.
    #[error("root element is not a dictionary")]
    RootNotDictionary,
    /// Torrent does not contain an info dictionary.
    #[error("no info dictionary")]
    NoInfo,
    /// Torrent does not contain a files list
    #[error("single file torrent")]
    NoFiles,
    #[error("file entry is not a dictionary")]
    NoFileDictionary,
    #[error("no path list")]
    NoPathList,
    #[error("invalid path part")]
    InvalidPathPart,
    #[error("value is not a string or bytes")]
    NotStringOrBytes,
    #[error("multiple root elements")]
    MultipleElements,
    /// A path element that is not valid UTF-8.
    #[error("{0}")]
    Path(AuditPathIssueKind),
    #[error("no name")]
    NoName,
    #[error("unknown path encoding")]
    UnknownPathEncoding,
    #[error("unknown name encoding")]
    UnknownNameEncoding,
    #[error("read file")]
    ReadFile,
    /// `name.utf-8` key is present but the wrong bencode type.
    ///
    /// OPS selects the variant regardless of type while libtorrent falls back
    /// to the legacy key, so the two clients diverge on the source value.
    #[error("name.utf-8 is invalid")]
    NameDivergence,
    /// `path.utf-8` key is present but the wrong bencode type.
    ///
    /// OPS selects the variant regardless of type while libtorrent falls back
    /// to the legacy key, so the two clients diverge on the source value.
    #[error("path.utf-8 is invalid")]
    PathDivergence,
    /// `name.utf-8` key is present but empty.
    ///
    /// The on-disk name definitely differs between clients: OPS uses the empty
    /// value while libtorrent falls back to the legacy `name`.
    #[error("name.utf-8 is empty")]
    NameEmpty,
    /// `path.utf-8` key is present but an empty list.
    ///
    /// The result definitely differs between clients: OPS uses the empty path
    /// while libtorrent rejects it.
    #[error("path.utf-8 is empty")]
    PathEmpty,
}

impl From<AuditIssueKind> for AuditIssue {
    fn from(kind: AuditIssueKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }
}

/// Category of a path-character problem found in a torrent file path.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, ThisError)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum AuditPathIssueKind {
    #[default]
    #[error("unknown")]
    Unknown,
    /// A path element that is not valid UTF-8.
    #[error("non-UTF-8")]
    NonUtf8,
    /// A path element containing restricted characters.
    #[error("restricted")]
    Restricted,
    /// A path element containing invisible or zero-width characters.
    #[error("invisible")]
    Invisible,
    /// A path element containing characters libtorrent strips on disk.
    ///
    /// The on-disk name produced by a client will not match the torrent path.
    #[error("libtorrent stripped")]
    LibtorrentStripped,
    /// A path element that is not a single safe path segment.
    #[error("unsafe")]
    UnsafeSegment,
    /// A path element that is not in Unicode NFC (canonical composed) form.
    ///
    /// Decomposed elements render identically to their composed form but differ
    /// byte-for-byte, causing on-disk path mismatches.
    #[error("decomposed (non-NFC)")]
    Decomposed,
    /// A path element whose file extension is lost when written to disk.
    ///
    /// A non-UTF-8 byte immediately before the extension is replaced with a
    /// single `_` that consumes the `.` separator, so the on-disk name loses
    /// its extension.
    #[error("extension breaking")]
    BrokenExtension,
    /// A path element containing directional formatting marks with no effect.
    ///
    /// Directional marks only reorder text when a right-to-left character is
    /// present, so a mark in a component with none is invisible and inert.
    #[error("unnecessary directional")]
    UnnecessaryDirectional,
}

impl AuditIssue {
    /// Human-readable one-line description for console output.
    pub(crate) fn render(&self, bb_code: bool) -> String {
        let AuditIssueKind::Path(kind) = self.kind else {
            return format!("  {}", self.kind);
        };
        let mut output = format!("  Contains {kind} characters");
        let Some(raw) = &self.raw else {
            return output;
        };
        let renderer = if bb_code {
            DiffRenderer::colored_bb_code()
        } else {
            DiffRenderer::colored()
        };
        if let Some(suggestions) = &self.suggestions {
            render_suggestions(&mut output, raw, suggestions, renderer);
        } else if matches!(kind, AuditPathIssueKind::BrokenExtension) {
            render_broken_extension(&mut output, raw, renderer);
        } else if matches!(
            kind,
            AuditPathIssueKind::Restricted
                | AuditPathIssueKind::Invisible
                | AuditPathIssueKind::LibtorrentStripped
                | AuditPathIssueKind::UnnecessaryDirectional
        ) && let Some(characters) = &self.sanitized
        {
            render_sanitized(&mut output, raw, characters);
        } else {
            render_fallback(&mut output, raw);
        }
        output
    }
}

const PAD: &str = "\n    ";

fn render_suggestions(
    output: &mut String,
    raw: &RawString,
    suggestions: &Vec<AuditSuggestion>,
    renderer: DiffRenderer,
) {
    let differ = Differ::new(renderer);
    for suggestion in suggestions {
        let diff = differ.execute(raw, &suggestion.value);
        write!(output, "{PAD}{}: {}", suggestion.encoding, diff).expect("should write");
    }
}

fn render_broken_extension(output: &mut String, raw: &RawString, renderer: DiffRenderer) {
    let on_disk = LibtorrentDecoder::decode(raw.as_bytes());
    let differ = Differ::new(renderer);
    let diff = differ.execute(raw, &on_disk);
    write!(output, "{PAD}{diff}").expect("should write");
}

fn render_sanitized(output: &mut String, raw: &RawString, characters: &HashSet<SanitizerChar>) {
    let mut value = raw.to_string_with_hex();
    for sanitized in characters {
        value = value.replace(
            sanitized.get_char(),
            &format!("<{}>", sanitized.to_string().to_uppercase())
                .red()
                .to_string(),
        );
    }
    write!(output, "{PAD}{}", value.dimmed()).expect("should write");
}

fn render_fallback(output: &mut String, raw: &RawString) {
    write!(output, "{PAD}{}", raw.to_string_with_hex().dimmed()).expect("should write");
}
