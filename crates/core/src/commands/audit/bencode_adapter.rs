use crate::prelude::*;
use encoding_rs::Encoding;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::Dictionary;

/// Typed accessors over a torrent's root bencode dictionary.
pub(crate) struct BencodeAdapter {
    root: Dictionary,
}

impl BencodeAdapter {
    /// Create a [`BencodeAdapter`] from a root dictionary.
    pub fn new(root: Dictionary) -> Self {
        Self { root }
    }

    /// Get the top-level `comment` field, if present and valid UTF-8.
    ///
    /// - Replace RED `.ch` extension with `.sh`
    pub fn get_comment(&self) -> Option<String> {
        let value = get_string(&self.root, "comment")?;
        if value.starts_with(RED_URL_CH) {
            Some(value.replacen(RED_URL_CH, RED_URL, 1))
        } else {
            Some(value)
        }
    }

    /// Get the top-level `comment` field, if present and valid UTF-8.
    pub fn get_source(&self) -> Option<String> {
        let info = self.get_info().ok()?;
        get_string(info, "source")
    }

    /// Get the top-level `encoding` field as an [`Encoding`], if a known label.
    pub fn get_encoding(&self) -> Option<&'static Encoding> {
        match self.root.get("encoding") {
            Some(BencodeElem::String(value)) => Encoding::for_label(value.as_bytes()),
            Some(BencodeElem::Bytes(bytes)) => Encoding::for_label(bytes),
            _ => None,
        }
    }

    /// Get the raw `info.name` value as a [`RawString`].
    pub fn get_name(&self) -> Result<RawString, AuditIssueKind> {
        let info = self.get_info()?;
        if let Some(bencode) = info.get("name.utf-8") {
            trace!("Using name.utf-8");
            let raw =
                RawString::try_from(bencode.clone()).map_err(|_| AuditIssueKind::NameDivergence)?;
            if raw.is_empty() {
                return Err(AuditIssueKind::NameEmpty);
            }
            return Ok(raw);
        }
        let bencode = info.get("name").ok_or(AuditIssueKind::NoName)?;
        match bencode {
            BencodeElem::String(_) | BencodeElem::Bytes(_) => RawString::try_from(bencode.clone()),
            _ => Err(AuditIssueKind::NoName),
        }
    }

    fn get_info(&self) -> Result<&Dictionary, AuditIssueKind> {
        let Some(BencodeElem::Dictionary(info)) = self.root.get("info") else {
            return Err(AuditIssueKind::NoInfo);
        };
        Ok(info)
    }

    /// Get the raw `info.files[].path` components as [`RawString`] lists.
    ///
    /// - Returns an empty list for single-file torrents with no `files` list
    pub fn get_paths(&self) -> Result<Vec<Vec<RawString>>, AuditIssueKind> {
        let mut output = Vec::new();
        let Some(BencodeElem::List(files)) = self.get_info()?.get("files") else {
            return Ok(output);
        };
        for file in files {
            let BencodeElem::Dictionary(file) = file else {
                return Err(AuditIssueKind::NoFileDictionary);
            };
            let parts = match file.get("path.utf-8") {
                Some(BencodeElem::List(parts)) => {
                    trace!("Using path.utf-8");
                    if parts.is_empty() {
                        return Err(AuditIssueKind::PathEmpty);
                    }
                    parts
                }
                Some(_) => return Err(AuditIssueKind::PathDivergence),
                None => {
                    let Some(BencodeElem::List(parts)) = file.get("path") else {
                        return Err(AuditIssueKind::NoPathList);
                    };
                    parts
                }
            };
            let mut path = Vec::new();
            for part in parts {
                let raw = RawString::try_from(part.clone())
                    .map_err(|_| AuditIssueKind::InvalidPathPart)?;
                path.push(raw);
            }
            output.push(path);
        }
        Ok(output)
    }
}

impl TryFrom<Vec<BencodeElem>> for BencodeAdapter {
    type Error = AuditIssueKind;

    fn try_from(elements: Vec<BencodeElem>) -> Result<Self, Self::Error> {
        if elements.is_empty() {
            return Err(AuditIssueKind::NoRootElement);
        }
        if elements.len() > 1 {
            return Err(AuditIssueKind::MultipleElements);
        }
        let Some(BencodeElem::Dictionary(dictionary)) = elements.into_iter().next() else {
            return Err(AuditIssueKind::RootNotDictionary);
        };
        Ok(BencodeAdapter::new(dictionary))
    }
}

fn get_string(dictionary: &Dictionary, key: &str) -> Option<String> {
    match dictionary.get(key) {
        Some(BencodeElem::String(value)) => Some(value.clone()),
        Some(BencodeElem::Bytes(bytes)) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}
