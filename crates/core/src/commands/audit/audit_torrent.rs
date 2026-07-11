use crate::prelude::*;

/// A `.torrent` file parsed into decoded name and path components.
pub struct AuditTorrent {
    /// Decoded torrent name from `info.name`.
    pub name: DecodedString,
    /// Decoded path components for each file, as `info.files[].path` lists.
    pub paths: Vec<Vec<DecodedString>>,
    /// Tracker `source` field.
    pub source: Option<String>,
    /// Tracker torrent id extracted from the `comment` URL, if any.
    pub id: Option<u32>,
    /// Tracker torrent URL from the `comment` field, if it is a recognized tracker URL.
    pub url: Option<String>,
}
