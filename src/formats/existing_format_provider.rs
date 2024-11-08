use std::collections::BTreeSet;

use crate::formats::ExistingFormat;
use gazelle_api::Torrent;
use regex::Regex;

pub struct ExistingFormatProvider;

impl ExistingFormatProvider {
    pub fn get(source_torrent: &Torrent, group_torrents: &[Torrent]) -> BTreeSet<ExistingFormat> {
        group_torrents
            .iter()
            .filter(|&other_torrent| is_same_release(source_torrent, other_torrent))
            .filter_map(ExistingFormat::from_torrent)
            .collect()
    }
}

/// Determine if [`source`] and [`target`] are the same release.
fn is_same_release(source: &Torrent, target: &Torrent) -> bool {
    target.remaster_title == source.remaster_title
        && target.remaster_record_label == source.remaster_record_label
        && target.media == source.media
        && is_equal_numeric(&target.remaster_catalogue_number, &source.remaster_catalogue_number)
}

fn is_equal_numeric(left: &str, right: &str) -> bool {
    if left == right {
        return true
    }
    let left = remove_zero_pad(left);
    let right = remove_zero_pad(right);
    left == right
}

fn remove_zero_pad(input: &str) -> String {
    let regex = Regex::new(r"^0*(\d+)$").expect("Regex should compile");
    regex.replace(input, "$1").to_string()  
}

#[cfg(test)]
mod tests {
    use gazelle_api::Torrent;
    use super::{is_same_release, remove_zero_pad};

    #[test]
    fn is_same_release_equal() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };

        // Act        
        // Assert
        assert!(is_same_release(&left, &right));
    }

    #[test]
    fn is_same_release_zero_padded() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "01234567".to_owned(),
            ..Torrent::default()
        };

        // Act        
        // Assert
        assert!(is_same_release(&left, &right));
    }

    #[test]
    fn is_same_release_invalid() {
        // Arrange
        let left = Torrent {
            remaster_catalogue_number: "1234567".to_owned(),
            ..Torrent::default()
        };
        let right = Torrent {
            remaster_catalogue_number: "0999999".to_owned(),
            ..Torrent::default()
        };

        // Act        
        // Assert
        assert!(!is_same_release(&left, &right));
    }

    #[test]
    fn remove_zero_pad_test() {
        assert_eq!(remove_zero_pad("01234"), "1234");
        assert_eq!(remove_zero_pad("1234"), "1234");
        assert_eq!(remove_zero_pad("001234"), "1234");
        assert_eq!(remove_zero_pad("9999990"), "9999990");
        assert_eq!(remove_zero_pad("09999990"), "9999990");
        assert_eq!(remove_zero_pad("-09999990"), "-09999990");
    }
}
