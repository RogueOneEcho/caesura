use crate::prelude::*;
use gazelle_api::{NewSourceUploadArtist, NewSourceUploadEdition, NewSourceUploadForm, UploadForm};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

const MUSIC_CATEGORY_ID: u8 = 0;

/// RED media values accepted in publish manifests.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PublishMedia {
    #[serde(rename = "CD")]
    Cd,
    #[serde(rename = "DVD")]
    Dvd,
    #[serde(rename = "Vinyl")]
    Vinyl,
    #[serde(rename = "Soundboard")]
    Soundboard,
    #[serde(rename = "SACD")]
    Sacd,
    #[serde(rename = "DAT")]
    Dat,
    #[serde(rename = "Cassette")]
    Cassette,
    #[serde(rename = "WEB")]
    Web,
    #[serde(rename = "Blu-Ray", alias = "Blu-ray")]
    BluRay,
}

impl PublishMedia {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cd => "CD",
            Self::Dvd => "DVD",
            Self::Vinyl => "Vinyl",
            Self::Soundboard => "Soundboard",
            Self::Sacd => "SACD",
            Self::Dat => "DAT",
            Self::Cassette => "Cassette",
            Self::Web => "WEB",
            Self::BluRay => "Blu-Ray",
        }
    }
}

impl Display for PublishMedia {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.as_str())
    }
}

/// Artist credit entry for new-group publish mode.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishArtist {
    /// Artist name. Gazelle resolves this to an existing artist or creates a new one.
    pub name: String,
    /// Gazelle artist importance index (`importance[]` in upload form).
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
    /// Gazelle numeric release type index.
    pub release_type: u8,
    pub media: PublishMedia,
    pub tags: Vec<String>,
    /// Group description in plain text.
    ///
    /// Backward compatibility: `album_desc` is also accepted.
    #[serde(alias = "album_desc")]
    pub album_description: String,
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
    pub media: PublishMedia,
    pub format: String,
    pub bitrate: String,
}

/// Publish target mode for either a new group or an existing one.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PublishGroup {
    NewGroup(PublishNewGroup),
    ExistingGroup(PublishExistingGroup),
}

/// Root publish manifest.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishManifest {
    /// Source directory containing FLAC files. Nested subdirectories are supported.
    pub source_path: PathBuf,
    /// Optional target path for the generated source torrent.
    pub torrent_path: Option<PathBuf>,
    /// Free-form notes to include in the generated release description.
    pub release_desc: String,
    /// Upload target metadata.
    pub group: PublishGroup,
}

/// Validation rule violation for a publish manifest.
#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
pub enum PublishValidationError {
    #[error("source_path is not a directory: {0}")]
    SourcePathNotDirectory(PathBuf),
    #[error("source_path could not be read: {path} ({error})")]
    SourcePathUnreadable { path: PathBuf, error: String },
    #[error("source_path contains no FLAC files: {0}")]
    SourcePathHasNoFlac(PathBuf),
    #[error("new_group.artists must contain at least one artist")]
    MissingArtists,
    #[error("new_group.artists contains an empty artist name")]
    EmptyArtistName,
    #[error("torrent_path parent directory does not exist: {0}")]
    TorrentPathParentMissing(PathBuf),
}

/// Aggregate publish manifest validation errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishValidationErrors(pub Vec<PublishValidationError>);

impl Display for PublishValidationErrors {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "publish manifest validation failed")?;
        for error in &self.0 {
            write!(formatter, "\n- {error}")?;
        }
        Ok(())
    }
}

impl Error for PublishValidationErrors {}

impl PublishGroup {
    #[must_use]
    pub fn source_title(&self) -> String {
        match self {
            PublishGroup::NewGroup(new_group) => {
                format!("{} {}", new_group.edition.format, new_group.edition.bitrate)
            }
            PublishGroup::ExistingGroup(existing_group) => {
                format!("{} {}", existing_group.format, existing_group.bitrate)
            }
        }
    }
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
    pub fn validate(&self) -> Result<(), Vec<PublishValidationError>> {
        let mut errors = Vec::new();
        if self.source_path.is_dir() {
            let flacs = DirectoryReader::new()
                .with_extension("flac")
                .read(&self.source_path);
            match flacs {
                Ok(files) => {
                    if files.is_empty() {
                        errors.push(PublishValidationError::SourcePathHasNoFlac(
                            self.source_path.clone(),
                        ));
                    }
                }
                Err(error) => {
                    errors.push(PublishValidationError::SourcePathUnreadable {
                        path: self.source_path.clone(),
                        error: error.to_string(),
                    });
                }
            }
        } else {
            errors.push(PublishValidationError::SourcePathNotDirectory(
                self.source_path.clone(),
            ));
        }

        if let PublishGroup::NewGroup(new_group) = &self.group {
            if new_group.artists.is_empty() {
                errors.push(PublishValidationError::MissingArtists);
            } else if new_group
                .artists
                .iter()
                .any(|artist| artist.name.trim().is_empty())
            {
                errors.push(PublishValidationError::EmptyArtistName);
            }
        }

        if let Some(torrent_path) = &self.torrent_path
            && let Some(parent) = torrent_path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.is_dir()
        {
            errors.push(PublishValidationError::TorrentPathParentMissing(
                parent.to_path_buf(),
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Build `gazelle_api` new-source upload payload.
    pub fn to_new_source_form(
        new_group: &PublishNewGroup,
        torrent_path: PathBuf,
        release_desc: String,
    ) -> NewSourceUploadForm {
        NewSourceUploadForm {
            path: torrent_path,
            category_id: MUSIC_CATEGORY_ID,
            title: new_group.title.clone(),
            year: new_group.year,
            release_type: new_group.release_type,
            media: new_group.media.to_string(),
            tags: new_group.tags.clone(),
            album_desc: new_group.album_description.clone(),
            release_desc,
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
    pub fn to_existing_group_form(
        existing_group: &PublishExistingGroup,
        torrent_path: PathBuf,
        release_desc: String,
    ) -> UploadForm {
        UploadForm {
            path: torrent_path,
            category_id: MUSIC_CATEGORY_ID,
            remaster_year: existing_group.remaster_year,
            remaster_title: existing_group.remaster_title.clone(),
            remaster_record_label: existing_group.remaster_record_label.clone(),
            remaster_catalogue_number: existing_group.remaster_catalogue_number.clone(),
            format: existing_group.format.clone(),
            bitrate: existing_group.bitrate.clone(),
            media: existing_group.media.to_string(),
            release_desc,
            group_id: existing_group.group_id,
        }
    }

    #[cfg(test)]
    pub fn mock_new(source_path: PathBuf) -> Self {
        Self {
            source_path,
            torrent_path: None,
            release_desc: "Release notes".to_owned(),
            group: PublishGroup::NewGroup(PublishNewGroup {
                title: "Album Title".to_owned(),
                year: 2024,
                release_type: 1,
                media: PublishMedia::Web,
                tags: vec!["electronic".to_owned(), "ambient".to_owned()],
                album_description: "Group description".to_owned(),
                request_id: Some(364_781),
                image: Some("https://example.com/cover.jpg".to_owned()),
                artists: vec![PublishArtist {
                    name: "Artist Name".to_owned(),
                    role: 1,
                }],
                edition: PublishNewGroupEdition {
                    unknown_release: false,
                    remaster: Some(true),
                    year: 2024,
                    title: "Digital".to_owned(),
                    record_label: "Label".to_owned(),
                    catalogue_number: "CAT-001".to_owned(),
                    format: "FLAC".to_owned(),
                    bitrate: "Lossless".to_owned(),
                },
            }),
        }
    }

    #[cfg(test)]
    pub fn mock_existing(source_path: PathBuf) -> Self {
        Self {
            source_path,
            torrent_path: None,
            release_desc: "Release notes".to_owned(),
            group: PublishGroup::ExistingGroup(PublishExistingGroup {
                group_id: 123_456,
                remaster_year: 2024,
                remaster_title: "Digital".to_owned(),
                remaster_record_label: "Label".to_owned(),
                remaster_catalogue_number: "CAT-001".to_owned(),
                media: PublishMedia::Web,
                format: "FLAC".to_owned(),
                bitrate: "Lossless".to_owned(),
            }),
        }
    }
}
