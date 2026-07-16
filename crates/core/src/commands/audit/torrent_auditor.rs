use crate::prelude::*;
use std::fs::read;
use unicode_normalization::is_nfc;

/// Inspect a `.torrent` file's paths for problems, tolerant of non-UTF-8 bytes.
#[injectable]
pub(crate) struct TorrentAuditor {
    options: Ref<AuditOptions>,
}

impl TorrentAuditor {
    /// Create a [`TorrentAuditor`] with the given options for testing.
    #[cfg(test)]
    pub(crate) fn new(options: AuditOptions) -> Self {
        Self {
            options: Ref::new(options),
        }
    }

    /// Create a [`TorrentAuditor`] with mock options for testing.
    #[cfg(test)]
    pub(crate) fn mock() -> Self {
        Self::new(AuditOptions::default())
    }

    /// Audit a `.torrent` file at `path` for problematic file paths.
    ///
    /// - Reports a [`AuditIssueKind::ReadFile`] issue when the file cannot be read
    pub(crate) fn execute_path(&self, path: &Path) -> AuditItem {
        let bytes = match read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Failed to read torrent file: {e}");
                return AuditItem {
                    path: path.to_path_buf(),
                    issues: Some(vec![AuditIssue::from(AuditIssueKind::ReadFile)]),
                    ..AuditItem::default()
                };
            }
        };
        let mut item = self.execute_bytes(&bytes);
        item.path = path.to_path_buf();
        item
    }

    /// Audit raw `.torrent` bytes for problematic file paths.
    pub(crate) fn execute_bytes(&self, bytes: &[u8]) -> AuditItem {
        let torrent = match TorrentParser::parse_bytes(bytes) {
            Ok(torrent) => torrent,
            Err(kind) => {
                return AuditItem {
                    issues: Some(vec![AuditIssue::from(kind)]),
                    ..AuditItem::default()
                };
            }
        };
        let mut issues = Vec::new();
        if !self.options.ignore_single_file && torrent.paths.is_empty() {
            issues.push(AuditIssue::from(AuditIssueKind::NoFiles));
        }
        if !self.options.ignore_non_utf8 {
            audit_non_utf8(&torrent.name, &mut issues);
            for parts in &torrent.paths {
                for part in parts {
                    audit_non_utf8(part, &mut issues);
                }
            }
        }
        if !self.options.ignore_lost_extension {
            audit_lost_extension(&torrent.paths, &mut issues);
        }
        if !issues.is_empty() {
            return AuditItem {
                issues: Some(issues),
                ..AuditItem::from(torrent)
            };
        }
        self.audit_value(&torrent.name, &mut issues);
        for parts in &torrent.paths {
            for part in parts {
                self.audit_value(part, &mut issues);
            }
        }
        AuditItem {
            issues: (!issues.is_empty()).then_some(issues),
            ..AuditItem::from(torrent)
        }
    }

    /// Classify a valid UTF-8 path component and append any [`AuditIssue`] found.
    ///
    /// - Checks for invisible characters, libtorrent-stripped characters, and unsafe
    ///   segments; overlaps are reported as separate issues
    /// - Skips any check disabled by the corresponding `ignore` option
    fn audit_value(&self, decoded: &DecodedString, issues: &mut Vec<AuditIssue>) {
        let value = match decoded {
            DecodedString::Known(value) => value,
            DecodedString::Suggestions(_, _) => {
                return;
            }
        };
        let restricted = Sanitizer::restricted().execute(value.to_owned());
        if !restricted.found.is_empty() {
            issues.push(AuditIssue {
                kind: AuditIssueKind::Path(AuditPathIssueKind::RestrictedChars),
                raw: Some(RawString::from(value)),
                sanitized: Some(restricted.found),
                ..AuditIssue::default()
            });
        }
        if !self.options.ignore_invisible {
            let invisible = Sanitizer::invisible().execute(value.to_owned());
            if !invisible.found.is_empty() {
                issues.push(AuditIssue {
                    kind: AuditIssueKind::Path(AuditPathIssueKind::InvisibleChars),
                    raw: Some(RawString::from(value)),
                    sanitized: Some(invisible.found),
                    ..AuditIssue::default()
                });
            }
        }
        if !self.options.ignore_directional && !value.contains_rtl() {
            let directional = Sanitizer::directional().execute(value.to_owned());
            if !directional.found.is_empty() {
                issues.push(AuditIssue {
                    kind: AuditIssueKind::Path(AuditPathIssueKind::UnnecessaryDirectional),
                    raw: Some(RawString::from(value)),
                    sanitized: Some(directional.found),
                    ..AuditIssue::default()
                });
            }
        }
        if !self.options.ignore_libtorrent {
            let libtorrent = Sanitizer::libtorrent().execute(value.to_owned());
            if !libtorrent.found.is_empty() {
                issues.push(AuditIssue {
                    kind: AuditIssueKind::Path(AuditPathIssueKind::LibtorrentStripped),
                    raw: Some(RawString::from(value)),
                    sanitized: Some(libtorrent.found),
                    ..AuditIssue::default()
                });
            }
        }
        if !self.options.ignore_unsafe && !is_single_safe_segment(value) {
            issues.push(AuditIssue {
                kind: AuditIssueKind::Path(AuditPathIssueKind::UnsafeSegment),
                raw: Some(RawString::from(value)),
                ..AuditIssue::default()
            });
        }
        if !self.options.ignore_nfd && !is_nfc(value) {
            issues.push(AuditIssue {
                kind: AuditIssueKind::Path(AuditPathIssueKind::Decomposed),
                raw: Some(RawString::from(value)),
                ..AuditIssue::default()
            });
        }
    }
}

/// Append a [`AuditIssue`] for any file whose extension is lost on disk.
///
/// - Checks only the final element of each path, the one carrying the extension
/// - A directory or valid-UTF-8 element can never lose its extension, so it is skipped
fn audit_lost_extension(paths: &[Vec<DecodedString>], issues: &mut Vec<AuditIssue>) {
    for parts in paths {
        let Some(DecodedString::Suggestions(raw, _)) = parts.last() else {
            continue;
        };
        if lost_extension(raw) {
            issues.push(AuditIssue {
                kind: AuditIssueKind::Path(AuditPathIssueKind::BrokenExtension),
                raw: Some(RawString::from(raw.clone())),
                ..AuditIssue::default()
            });
        }
    }
}

/// Detect a file extension lost when the raw name is written to disk.
///
/// - Returns `true` when the predicted on-disk name drops the raw extension
/// - Returns `false` when there is no extension or it survives decoding
fn lost_extension(raw: &[u8]) -> bool {
    let Some(extension) = get_extension(raw) else {
        return false;
    };
    let extension = String::from_utf8_lossy(extension);
    let libtorrent = LibtorrentDecoder::decode(raw);
    !libtorrent.ends_with(extension.as_ref())
}

fn get_extension(raw: &[u8]) -> Option<&[u8]> {
    let index = raw.iter().rposition(|&byte| byte == b'.')?;
    raw.get(index..)
}

/// Append a [`AuditIssue`] when `decoded` carries non-UTF-8 decoding suggestions.
fn audit_non_utf8(decoded: &DecodedString, issues: &mut Vec<AuditIssue>) {
    if let DecodedString::Suggestions(raw, suggestions) = decoded {
        issues.push(AuditIssue {
            kind: AuditIssueKind::Path(AuditPathIssueKind::NonUtf8),
            raw: Some(RawString::Bytes(raw.clone())),
            suggestions: Some(suggestions.clone()),
            ..AuditIssue::default()
        });
    }
}
