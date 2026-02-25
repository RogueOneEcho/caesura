use crate::prelude::*;
use gazelle_api::{NewSourceUploadArtist, NewSourceUploadEdition, NewSourceUploadForm, UploadForm};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

/// Supported publish modes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishMode {
    NewGroup,
    ExistingGroup,
}

/// Artist credit entry for new-group publish mode.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishArtist {
    pub name: String,
    pub role: u8,
}

/// Edition metadata for new-group publish mode.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishNewGroupEdition {
    pub unknown_release: bool,
    pub remaster: Option<bool>,
    pub year: u16,
    pub title: String,
    pub record_label: String,
    pub catalogue_number: String,
    pub format: String,
    pub bitrate: String,
}

/// New-group publish manifest section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishNewGroup {
    pub title: String,
    pub year: u16,
    pub release_type: u8,
    pub media: String,
    pub tags: Vec<String>,
    pub album_desc: String,
    pub request_id: Option<u32>,
    pub image: Option<String>,
    pub artists: Vec<PublishArtist>,
    pub edition: PublishNewGroupEdition,
}

/// Existing-group publish manifest section.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishExistingGroup {
    pub group_id: u32,
    pub remaster_year: u16,
    pub remaster_title: String,
    pub remaster_record_label: String,
    pub remaster_catalogue_number: String,
    pub media: String,
    pub format: String,
    pub bitrate: String,
}

/// Root publish manifest.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishManifest {
    pub source_path: PathBuf,
    pub torrent_path: Option<PathBuf>,
    pub manual_checks_ack: bool,
    #[serde(default)]
    pub dry_run: bool,
    pub mode: PublishMode,
    pub release_desc: String,
    pub new_group: Option<PublishNewGroup>,
    pub existing_group: Option<PublishExistingGroup>,
}

impl PublishManifest {
    /// Parse a publish manifest from a YAML file.
    pub fn read(path: &Path) -> Result<Self, IoError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader)
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e.to_string()))
    }

    /// Validate manifest structure and safety prerequisites.
    pub fn validate(&self) -> Result<(), IoError> {
        if !self.manual_checks_ack {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                "manual_checks_ack must be true",
            ));
        }
        if !self.source_path.is_dir() {
            return Err(IoError::new(
                ErrorKind::NotFound,
                format!(
                    "source_path is not a directory: {}",
                    self.source_path.display()
                ),
            ));
        }
        let flacs = DirectoryReader::new()
            .with_extension("flac")
            .read(&self.source_path)
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e.to_string()))?;
        if flacs.is_empty() {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                format!(
                    "source_path contains no FLAC files: {}",
                    self.source_path.display()
                ),
            ));
        }
        if self.new_group.is_some() == self.existing_group.is_some() {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                "exactly one of new_group or existing_group must be present",
            ));
        }
        match self.mode {
            PublishMode::NewGroup => {
                if self.new_group.is_none() {
                    return Err(IoError::new(
                        ErrorKind::InvalidInput,
                        "mode is new_group but new_group section is missing",
                    ));
                }
            }
            PublishMode::ExistingGroup => {
                if self.existing_group.is_none() {
                    return Err(IoError::new(
                        ErrorKind::InvalidInput,
                        "mode is existing_group but existing_group section is missing",
                    ));
                }
            }
        }
        if let Some(torrent_path) = &self.torrent_path
            && let Some(parent) = torrent_path.parent()
            && !parent.is_dir()
        {
            return Err(IoError::new(
                ErrorKind::NotFound,
                format!(
                    "torrent_path parent directory does not exist: {}",
                    parent.display()
                ),
            ));
        }
        Ok(())
    }

    /// Build `gazelle_api` new-source upload payload.
    pub fn to_new_source_form(&self, torrent_path: PathBuf) -> NewSourceUploadForm {
        let new_group = self
            .new_group
            .as_ref()
            .expect("new_group should be present after validation");
        NewSourceUploadForm {
            path: torrent_path,
            category_id: 0,
            title: new_group.title.clone(),
            year: new_group.year,
            release_type: new_group.release_type,
            media: new_group.media.clone(),
            tags: new_group.tags.clone(),
            album_desc: new_group.album_desc.clone(),
            release_desc: self.release_desc.clone(),
            request_id: new_group.request_id,
            image: new_group.image.clone(),
            edition: NewSourceUploadEdition {
                unknown_release: new_group.edition.unknown_release,
                remaster: new_group.edition.remaster,
                year: new_group.edition.year,
                title: new_group.edition.title.clone(),
                record_label: new_group.edition.record_label.clone(),
                catalogue_number: new_group.edition.catalogue_number.clone(),
                format: new_group.edition.format.clone(),
                bitrate: new_group.edition.bitrate.clone(),
            },
            artists: new_group
                .artists
                .iter()
                .map(|artist| NewSourceUploadArtist {
                    name: artist.name.clone(),
                    role: artist.role,
                })
                .collect(),
        }
    }

    /// Build `gazelle_api` existing-group upload payload.
    pub fn to_existing_group_form(&self, torrent_path: PathBuf) -> UploadForm {
        let existing_group = self
            .existing_group
            .as_ref()
            .expect("existing_group should be present after validation");
        UploadForm {
            path: torrent_path,
            category_id: 0,
            remaster_year: existing_group.remaster_year,
            remaster_title: existing_group.remaster_title.clone(),
            remaster_record_label: existing_group.remaster_record_label.clone(),
            remaster_catalogue_number: existing_group.remaster_catalogue_number.clone(),
            format: existing_group.format.clone(),
            bitrate: existing_group.bitrate.clone(),
            media: existing_group.media.clone(),
            release_desc: self.release_desc.clone(),
            group_id: existing_group.group_id,
        }
    }
}
