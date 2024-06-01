use di::Ref;
use rogue_oxide::formats::{ExistingFormat, SourceFormat, TargetFormat, TargetFormatProvider};
use rogue_oxide::options::TranscodeOptions;
use std::collections::HashSet;

#[test]
fn from_flac24_without_existing() {
    // Arrange
    let source = SourceFormat::Flac24;
    let target = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0];
    let existing = HashSet::from([ExistingFormat::Flac24, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(result, [TargetFormat::Flac, TargetFormat::V0]);
}

#[test]
fn from_flac_without_existing() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0];
    let existing = HashSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(result, [TargetFormat::V0]);
}

#[test]
fn from_flac24_with_existing() {
    // Arrange
    let source = SourceFormat::Flac24;
    let target = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0];
    let source_format = ExistingFormat::Flac;
    let existing = HashSet::from([source_format]);
    let provider = create_provider(target, true);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(
        result,
        [TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]
    );
}

#[test]
fn from_flac_with_existing() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0];
    let source_format = ExistingFormat::Flac;
    let existing = HashSet::from([source_format]);
    let provider = create_provider(target, true);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(result, [TargetFormat::_320, TargetFormat::V0]);
}

#[test]
fn from_flac_applies_allowed() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = vec![TargetFormat::_320, TargetFormat::V0];
    let existing = HashSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(result, [TargetFormat::V0]);
}

#[test]
fn from_flac_applies_allowed_none() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = vec![TargetFormat::_320];
    let existing = HashSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    assert_eq!(result, []);
}

fn create_provider(target: Vec<TargetFormat>, allow_existing: bool) -> TargetFormatProvider {
    TargetFormatProvider {
        options: Ref::new(TranscodeOptions {
            target: Some(target),
            allow_existing: Some(allow_existing),
            ..TranscodeOptions::default()
        }),
    }
}
