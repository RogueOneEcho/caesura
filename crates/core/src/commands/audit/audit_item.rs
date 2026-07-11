use crate::prelude::*;

/// A `.torrent` file previously inspected by the `audit` command.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct AuditItem {
    /// Absolute path to the `.torrent` file.
    pub path: PathBuf,
    /// Torrent name, for human-readable listings.
    pub name: Option<String>,
    /// Torrent `source`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Tracker torrent id from the `comment` URL, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Tracker torrent URL from the `comment` field, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Path problems found, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<Vec<AuditIssue>>,
}

impl From<AuditIssueKind> for AuditItem {
    fn from(value: AuditIssueKind) -> Self {
        Self {
            issues: Some(vec![AuditIssue {
                kind: value,
                ..AuditIssue::default()
            }]),
            ..Self::default()
        }
    }
}

impl From<AuditTorrent> for AuditItem {
    fn from(torrent: AuditTorrent) -> Self {
        Self {
            name: Some(torrent.name.to_string()),
            source: torrent.source,
            id: torrent.id,
            url: torrent.url.clone(),
            ..AuditItem::default()
        }
    }
}

impl AuditItem {
    /// Human-readable one-line label for console output.
    ///
    /// - Renders the torrent name as a hyperlink when a URL is known
    /// - Falls back to the file name when the torrent name is absent
    pub fn render(&self, bb_code: bool) -> String {
        if bb_code && let Some(url) = &self.url {
            return url.dimmed().hyperlink(url).clone();
        }
        let path = self
            .path
            .file_name()
            .expect("should have a file name")
            .display()
            .to_string()
            .dimmed();
        let Some(name) = &self.name else {
            return format!("{path}");
        };
        let name = name.cyan().dimmed();
        let name = match &self.url {
            Some(url) => name.hyperlink(url),
            None => name.to_string(),
        };
        if let Some(source) = &self.source
            && let Some(id) = &self.id
        {
            format!("{} {} {name}", source.dimmed(), id.to_string().dimmed())
        } else {
            format!("{name} {path}")
        }
    }

    /// Are any issues of specified `kind`.
    #[cfg(test)]
    pub fn has_kind(&self, kind: AuditIssueKind) -> bool {
        self.issues.iter().flatten().any(|issue| issue.kind == kind)
    }

    /// Are any issues of specified `kind`.
    #[cfg(test)]
    pub fn has_path_kind(&self, kind: AuditPathIssueKind) -> bool {
        self.has_kind(AuditIssueKind::Path(kind))
    }
}
