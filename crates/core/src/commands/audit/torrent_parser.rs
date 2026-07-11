use crate::prelude::*;
use cesu8::from_cesu8;
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, WINDOWS_1252};
use lava_torrent::bencode::BencodeElem;

/// Parse `.torrent` bytes into an [`AuditTorrent`], tolerant of non-UTF-8 paths.
pub struct TorrentParser;

impl TorrentParser {
    /// Parse raw `.torrent` bytes into an [`AuditTorrent`].
    ///
    /// - Bencode-decodes the bytes and validates the root dictionary
    /// - Returns the [`AuditIssueKind`] describing the first failure
    pub fn parse_bytes(bytes: &[u8]) -> Result<AuditTorrent, AuditIssueKind> {
        let result = BencodeElem::from_bytes(bytes);
        match result {
            Ok(elements) => {
                let adapter = BencodeAdapter::try_from(elements)?;
                Self::parse(adapter)
            }
            Err(e) => {
                warn!("Failed to parse bytes: {e:?}");
                Err(AuditIssueKind::Parse)
            }
        }
    }

    /// Audit bencode elements.
    ///
    /// - Parses with `BencodeElem::from_bytes` so non-UTF-8 path elements surface as `Bytes`
    /// - Reads the top-level `encoding` field, if any, as a decoding hint
    /// - Walks `info.name` and every `info.files[].path` element
    /// - Detects an encoding once over all non-UTF-8 bytes, then decodes each element
    pub(crate) fn parse(torrent: BencodeAdapter) -> Result<AuditTorrent, AuditIssueKind> {
        let name = torrent.get_name()?;
        let paths = torrent.get_paths()?;
        let source = torrent.get_source();
        let comment = torrent.get_comment();
        let id = comment
            .as_deref()
            .and_then(|comment| get_torrent_id_from_url(comment).ok());
        let url = id.and(comment);
        if let Ok(name) = name.try_to_string()
            && let Some(paths) = paths_to_known(&paths)
        {
            return Ok(AuditTorrent {
                name: DecodedString::Known(name),
                paths,
                source,
                id,
                url,
            });
        }
        let (encoding, kind) = match torrent.get_encoding() {
            None => {
                trace!("Detecting encoding");
                let encoding = detect_encoding(&name, &paths);
                (encoding, AuditSuggestionKind::Chardetng)
            }
            Some(encoding) => {
                trace!("Using stored encoding");
                (encoding, AuditSuggestionKind::EncodingField)
            }
        };
        let name = decode(name, encoding, kind).ok_or(AuditIssueKind::UnknownNameEncoding)?;
        let paths = decode_paths(paths, encoding, kind)?;
        Ok(AuditTorrent {
            name,
            paths,
            source,
            id,
            url,
        })
    }
}

fn paths_to_known(paths: &Vec<Vec<RawString>>) -> Option<Vec<Vec<DecodedString>>> {
    let mut output = Vec::new();
    for path in paths {
        let mut current = Vec::new();
        for part in path {
            if let Ok(value) = part.try_to_string() {
                current.push(DecodedString::Known(value));
            } else {
                return None;
            }
        }
        output.push(current);
    }
    Some(output)
}

fn detect_encoding(name: &RawString, paths: &Vec<Vec<RawString>>) -> &'static Encoding {
    let mut sample = Vec::new();
    if let RawString::Bytes(bytes) = name {
        sample.extend_from_slice(bytes);
    }
    for path in paths {
        for part in path {
            if let RawString::Bytes(bytes) = part {
                sample.extend_from_slice(bytes);
            }
        }
    }
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(&sample, true);
    detector.guess(None, Utf8Detection::Allow)
}

fn decode(
    raw: RawString,
    encoding: &'static Encoding,
    kind: AuditSuggestionKind,
) -> Option<DecodedString> {
    let bytes = match raw {
        RawString::String(value) => return Some(DecodedString::Known(value)),
        RawString::Bytes(bytes) => bytes,
    };
    decode_bytes(bytes, encoding, kind)
}

fn decode_paths(
    paths: Vec<Vec<RawString>>,
    encoding: &'static Encoding,
    kind: AuditSuggestionKind,
) -> Result<Vec<Vec<DecodedString>>, AuditIssueKind> {
    let mut output = Vec::new();
    for parts in paths {
        let mut path = Vec::new();
        for part in parts {
            if let Some(decoded) = decode(part, encoding, kind) {
                path.push(decoded);
            } else {
                return Err(AuditIssueKind::UnknownPathEncoding);
            }
        }
        output.push(path);
    }
    Ok(output)
}

fn decode_bytes(
    bytes: Vec<u8>,
    encoding: &'static Encoding,
    kind: AuditSuggestionKind,
) -> Option<DecodedString> {
    let mut suggestions = Vec::new();
    if let Some(suggestion) = try_cesu8(&bytes) {
        suggestions.push(suggestion);
    }
    if let Some(decoded) = encoding.decode_without_bom_handling_and_without_replacement(&bytes) {
        suggestions.push(AuditSuggestion {
            encoding: encoding.name().to_owned(),
            kind,
            value: decoded.into_owned(),
        });
    }
    let encoding_name = encoding.name();
    if encoding_name.starts_with("windows")
        && encoding_name != "windows-1252"
        && let Some(suggestion) = try_windows_1252(&bytes)
    {
        suggestions.push(suggestion);
    }
    if suggestions.is_empty() {
        None
    } else {
        Some(DecodedString::Suggestions(bytes, suggestions))
    }
}

/// Decode non-UTF-8 `bytes` as CESU-8 into a [`AuditSuggestion`], when valid.
///
/// - Returns [`None`] unless the bytes are entirely valid CESU-8
/// - Callers only reach this for bytes that already failed UTF-8 decoding, so a
///   successful decode always recovers a surrogate-paired astral character
fn try_cesu8(bytes: &[u8]) -> Option<AuditSuggestion> {
    let decoded = from_cesu8(bytes).ok()?;
    Some(AuditSuggestion {
        encoding: "CESU-8".to_owned(),
        kind: AuditSuggestionKind::Cesu8,
        value: decoded.into_owned(),
    })
}

fn try_windows_1252(bytes: &[u8]) -> Option<AuditSuggestion> {
    let decoded = WINDOWS_1252.decode_without_bom_handling_and_without_replacement(bytes)?;
    Some(AuditSuggestion {
        encoding: WINDOWS_1252.name().to_owned(),
        kind: AuditSuggestionKind::Windows1252,
        value: decoded.into_owned(),
    })
}
