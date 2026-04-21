use crate::testing_prelude::*;

#[test]
fn existing_format_provider_get_excludes_less_specific() {
    // Arrange
    let source = Torrent {
        id: 1,
        remaster_record_label: String::new(),
        ..Torrent::mock()
    };
    let existing_v0 = Torrent {
        id: 2,
        format: Format::MP3,
        encoding: Quality::V0,
        ..Torrent::mock()
    };
    let group_torrents = vec![source.clone(), existing_v0];
    let provider = create_provider(false);

    // Act
    let output = provider.get(&source, &group_torrents);

    // Assert
    assert!(output.contains(&ExistingFormat::V0));
}

#[test]
fn existing_format_provider_get_allows_less_specific_when_option_set() {
    // Arrange
    let source = Torrent {
        id: 1,
        remaster_record_label: String::new(),
        ..Torrent::mock()
    };
    let existing_v0 = Torrent {
        id: 2,
        format: Format::MP3,
        encoding: Quality::V0,
        ..Torrent::mock()
    };
    let group_torrents = vec![source.clone(), existing_v0];
    let provider = create_provider(true);

    // Act
    let output = provider.get(&source, &group_torrents);

    // Assert
    assert!(!output.contains(&ExistingFormat::V0));
}

#[test]
fn existing_format_provider_get_includes_exact_match() {
    // Arrange
    let source = Torrent {
        id: 1,
        ..Torrent::mock()
    };
    let existing_v0 = Torrent {
        id: 2,
        format: Format::MP3,
        encoding: Quality::V0,
        ..Torrent::mock()
    };
    let group_torrents = vec![source.clone(), existing_v0];
    let provider = create_provider(true);

    // Act
    let output = provider.get(&source, &group_torrents);

    // Assert
    assert!(output.contains(&ExistingFormat::V0));
}

#[test]
fn existing_format_provider_get_source_more_specific() {
    // Arrange
    let source = Torrent {
        id: 1,
        ..Torrent::mock()
    };
    let existing_v0 = Torrent {
        id: 2,
        format: Format::MP3,
        encoding: Quality::V0,
        remaster_record_label: String::new(),
        remaster_catalogue_number: String::new(),
        ..Torrent::mock()
    };
    let group_torrents = vec![source.clone(), existing_v0];
    let provider = create_provider(false);

    // Act
    let output = provider.get(&source, &group_torrents);

    // Assert
    assert!(!output.contains(&ExistingFormat::V0));
}

fn create_provider(allow_less_specific: bool) -> ExistingFormatProvider {
    ExistingFormatProvider {
        options: Ref::new(TargetOptions {
            allow_less_specific,
            ..TargetOptions::default()
        }),
    }
}
