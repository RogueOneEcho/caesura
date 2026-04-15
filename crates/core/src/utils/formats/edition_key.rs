use gazelle_api::{Media, Torrent};

/// Edition identity for grouping torrents of the same release.
///
/// Two torrents with the same `EditionKey` are considered the same edition.
/// Uses zero-pad normalization on catalogue numbers so `"01234"` and `"1234"`
/// produce the same key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EditionKey {
    remaster_title: String,
    remaster_record_label: String,
    remaster_catalogue_number: String,
    media: Media,
}

impl EditionKey {
    /// Create an [`EditionKey`] from a [`Torrent`].
    #[must_use]
    pub(crate) fn from_torrent(torrent: &Torrent) -> Self {
        Self {
            remaster_title: torrent.remaster_title.clone(),
            remaster_record_label: torrent.remaster_record_label.clone(),
            remaster_catalogue_number: remove_zero_pad(&torrent.remaster_catalogue_number),
            media: torrent.media.clone(),
        }
    }

    /// Check whether this edition key is a less specific match.
    ///
    /// Returns `true` when `remaster_title` and `media` match exactly, at
    /// least one field is less specific (empty on self but populated on other),
    /// and no populated fields conflict.
    ///
    /// Exact matches return `false` (those are handled by `PartialEq`).
    #[must_use]
    pub(crate) fn is_less_specific_than(&self, other: &EditionKey) -> bool {
        if self.remaster_title != other.remaster_title || self.media != other.media {
            return false;
        }
        let label = self.remaster_record_label == other.remaster_record_label;
        let number = self.remaster_catalogue_number == other.remaster_catalogue_number;
        if self.remaster_record_label.is_empty() && number {
            return true;
        }
        if self.remaster_catalogue_number.is_empty() && label {
            return true;
        }
        false
    }

    /// Create a mock [`EditionKey`] for testing.
    #[cfg(test)]
    pub(crate) fn mock() -> Self {
        Self {
            remaster_title: "Test Edition".to_owned(),
            remaster_record_label: "Test Label".to_owned(),
            remaster_catalogue_number: "TEST-001".to_owned(),
            media: Media::CD,
        }
    }
}

/// Remove leading zeros from a string.
fn remove_zero_pad(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let trimmed = input.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gazelle_api::Torrent;

    #[test]
    fn edition_key_same_torrent_fields() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };

        // Act & Assert
        assert_eq!(
            EditionKey::from_torrent(&left),
            EditionKey::from_torrent(&right)
        );
    }

    #[test]
    fn edition_key_zero_padded_catalogue_numbers_match() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "01234567".to_owned(),
            ..Torrent::default()
        };

        // Act & Assert
        assert_eq!(
            EditionKey::from_torrent(&left),
            EditionKey::from_torrent(&right)
        );
    }

    #[test]
    fn edition_key_different_catalogue_numbers_differ() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "0999999".to_owned(),
            ..Torrent::default()
        };

        // Act & Assert
        assert_ne!(
            EditionKey::from_torrent(&left),
            EditionKey::from_torrent(&right)
        );
    }

    #[test]
    fn edition_key_different_media_differ() {
        // Arrange
        let left = Torrent {
            media: Media::CD,
            ..Torrent::default()
        };
        let right = Torrent {
            media: Media::WEB,
            ..Torrent::default()
        };

        // Act & Assert
        assert_ne!(
            EditionKey::from_torrent(&left),
            EditionKey::from_torrent(&right)
        );
    }

    #[test]
    fn edition_key_different_title_differ() {
        // Arrange
        let left = Torrent {
            remaster_title: "Deluxe".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_title: "Standard".to_owned(),
            ..Torrent::default()
        };

        // Act & Assert
        assert_ne!(
            EditionKey::from_torrent(&left),
            EditionKey::from_torrent(&right)
        );
    }

    #[test]
    fn edition_key_is_less_specific_than_missing_label() {
        // Arrange
        let source = EditionKey {
            remaster_record_label: String::new(),
            ..EditionKey::mock()
        };
        let existing = EditionKey::mock();

        // Act & Assert
        assert!(source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_missing_catalogue() {
        // Arrange
        let source = EditionKey {
            remaster_catalogue_number: String::new(),
            ..EditionKey::mock()
        };
        let existing = EditionKey::mock();

        // Act & Assert
        assert!(source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_source_more_specific() {
        // Arrange
        let source = EditionKey::mock();
        let existing = EditionKey {
            remaster_record_label: String::new(),
            remaster_catalogue_number: String::new(),
            ..EditionKey::mock()
        };

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_different_values() {
        // Arrange
        let source = EditionKey::mock();
        let existing = EditionKey {
            remaster_record_label: "Other Label".to_owned(),
            remaster_catalogue_number: "OTHER-001".to_owned(),
            ..EditionKey::mock()
        };

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_cross_field() {
        // Arrange
        let source = EditionKey {
            remaster_catalogue_number: String::new(),
            ..EditionKey::mock()
        };
        let existing = EditionKey {
            remaster_record_label: String::new(),
            ..EditionKey::mock()
        };

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_exact_match() {
        // Arrange
        let source = EditionKey::mock();
        let existing = EditionKey::mock();

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_than_different_media() {
        // Arrange
        let source = EditionKey {
            remaster_record_label: String::new(),
            ..EditionKey::mock()
        };
        let existing = EditionKey {
            media: Media::WEB,
            ..EditionKey::mock()
        };

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn edition_key_is_less_specific_mismatch() {
        // Arrange
        let source = EditionKey {
            remaster_record_label: String::new(),
            remaster_catalogue_number: "TEST-002".to_owned(),
            ..EditionKey::mock()
        };
        let existing = EditionKey::mock();

        // Act & Assert
        assert!(!source.is_less_specific_than(&existing));
    }

    #[test]
    fn remove_zero_pad_edge_cases() {
        assert_eq!(remove_zero_pad("01234"), "1234");
        assert_eq!(remove_zero_pad("1234"), "1234");
        assert_eq!(remove_zero_pad("001234"), "1234");
        assert_eq!(remove_zero_pad("9999990"), "9999990");
        assert_eq!(remove_zero_pad("09999990"), "9999990");
        assert_eq!(remove_zero_pad("-09999990"), "-09999990");
        assert_eq!(remove_zero_pad("000"), "0");
        assert_eq!(remove_zero_pad("0"), "0");
        assert_eq!(remove_zero_pad("0abc"), "abc");
        assert_eq!(remove_zero_pad(""), "");
    }
}
