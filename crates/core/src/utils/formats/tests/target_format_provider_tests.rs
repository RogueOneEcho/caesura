use crate::testing_prelude::*;

#[test]
fn from_flac24_without_existing() {
    // Arrange
    let source = SourceFormat::Flac24;
    let target = BTreeSet::from([TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]);
    let existing = BTreeSet::from([ExistingFormat::Flac24, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([TargetFormat::Flac, TargetFormat::V0]);
    assert_eq!(result, expected);
}

#[test]
fn from_flac_without_existing() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = BTreeSet::from([TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]);
    let existing = BTreeSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([TargetFormat::V0]);
    assert_eq!(result, expected);
}

#[test]
fn from_flac24_with_existing() {
    // Arrange
    let source = SourceFormat::Flac24;
    let target = BTreeSet::from([TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]);
    let source_format = ExistingFormat::Flac;
    let existing = BTreeSet::from([source_format]);
    let provider = create_provider(target, true);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]);
    assert_eq!(result, expected);
}

#[test]
fn from_flac_with_existing() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = BTreeSet::from([TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]);
    let source_format = ExistingFormat::Flac;
    let existing = BTreeSet::from([source_format]);
    let provider = create_provider(target, true);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([TargetFormat::_320, TargetFormat::V0]);
    assert_eq!(result, expected);
}

#[test]
fn from_flac_applies_allowed() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = BTreeSet::from([TargetFormat::_320, TargetFormat::V0]);
    let existing = BTreeSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([TargetFormat::V0]);
    assert_eq!(result, expected);
}

#[test]
fn from_flac_applies_allowed_none() {
    // Arrange
    let source = SourceFormat::Flac;
    let target = BTreeSet::from([TargetFormat::_320]);
    let existing = BTreeSet::from([ExistingFormat::Flac, ExistingFormat::_320]);
    let provider = create_provider(target, false);

    // Act
    let result = provider.get(source, &existing);

    // Assert
    let expected = BTreeSet::from([]);
    assert_eq!(result, expected);
}

fn create_provider(target: BTreeSet<TargetFormat>, allow_existing: bool) -> TargetFormatProvider {
    TargetFormatProvider {
        options: Ref::new(TargetOptions {
            target: target.iter().copied().collect(),
            allow_existing,
            sox_random_dither: false,
        }),
    }
}
