use gazelle_api::Torrent;

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
    media: String,
}

impl EditionKey {
    /// Create an [`EditionKey`] from a [`Torrent`].
    #[must_use]
    pub(crate) fn from_torrent(torrent: &Torrent) -> Self {
        Self {
            remaster_title: torrent.remaster_title.clone(),
            remaster_record_label: torrent.remaster_record_label.clone(),
            remaster_catalogue_number: remove_zero_pad(&torrent.remaster_catalogue_number),
            media: torrent.media.to_string(),
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
    use gazelle_api::{Media, Torrent};

    #[test]
    #[expect(non_snake_case, reason = "double underscore test qualifier convention")]
    fn edition_key__same_torrent_fields() {
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
    #[expect(non_snake_case, reason = "double underscore test qualifier convention")]
    fn edition_key__zero_padded_catalogue_numbers_match() {
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
    #[expect(non_snake_case, reason = "double underscore test qualifier convention")]
    fn edition_key__different_catalogue_numbers_differ() {
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
    #[expect(non_snake_case, reason = "double underscore test qualifier convention")]
    fn edition_key__different_media_differ() {
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
    #[expect(non_snake_case, reason = "double underscore test qualifier convention")]
    fn edition_key__different_title_differ() {
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
