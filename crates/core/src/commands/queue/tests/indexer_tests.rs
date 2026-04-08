#![expect(non_snake_case, reason = "double underscore test qualifier convention")]

use crate::testing_prelude::*;

#[test]
fn indexer_deserialize__red_lowercase() {
    let indexer: Indexer = serde_yaml::from_str("red").expect("should deserialize");
    assert_eq!(indexer, Indexer::Red);
}

#[test]
fn indexer_deserialize__red_uppercase() {
    let indexer: Indexer = serde_yaml::from_str("RED").expect("should deserialize");
    assert_eq!(indexer, Indexer::Red);
}

#[test]
fn indexer_deserialize__red_mixed_case() {
    let indexer: Indexer = serde_yaml::from_str("Red").expect("should deserialize");
    assert_eq!(indexer, Indexer::Red);
}

#[test]
fn indexer_deserialize__ops_lowercase() {
    let indexer: Indexer = serde_yaml::from_str("ops").expect("should deserialize");
    assert_eq!(indexer, Indexer::Ops);
}

#[test]
fn indexer_deserialize__pth_lowercase() {
    let indexer: Indexer = serde_yaml::from_str("pth").expect("should deserialize");
    assert_eq!(indexer, Indexer::Pth);
}

#[test]
fn indexer_deserialize__unknown_normalizes_to_lowercase() {
    let indexer: Indexer = serde_yaml::from_str("ABCdefGHI").expect("should deserialize");
    assert_eq!(indexer, Indexer::Other("abcdefghi".to_owned()));
}

#[test]
fn indexer_deserialize__empty_string() {
    let indexer: Indexer = serde_yaml::from_str("''").expect("should deserialize");
    assert_eq!(indexer, Indexer::Other(String::new()));
}

#[test]
fn indexer_serialize__red() {
    let yaml = serde_yaml::to_string(&Indexer::Red).expect("should serialize");
    assert_eq!(yaml.trim(), "red");
}

#[test]
fn indexer_serialize__ops() {
    let yaml = serde_yaml::to_string(&Indexer::Ops).expect("should serialize");
    assert_eq!(yaml.trim(), "ops");
}

#[test]
fn indexer_serialize__pth() {
    let yaml = serde_yaml::to_string(&Indexer::Pth).expect("should serialize");
    assert_eq!(yaml.trim(), "pth");
}

#[test]
fn indexer_serialize__other() {
    let yaml = serde_yaml::to_string(&Indexer::Other("abc".to_owned())).expect("should serialize");
    assert_eq!(yaml.trim(), "abc");
}

#[test]
fn indexer_round_trip__red() {
    let yaml = serde_yaml::to_string(&Indexer::Red).expect("should serialize");
    let indexer: Indexer = serde_yaml::from_str(&yaml).expect("should deserialize");
    assert_eq!(indexer, Indexer::Red);
}

#[test]
fn indexer_round_trip__other() {
    let original = Indexer::Other("abc".to_owned());
    let yaml = serde_yaml::to_string(&original).expect("should serialize");
    let indexer: Indexer = serde_yaml::from_str(&yaml).expect("should deserialize");
    assert_eq!(indexer, original);
}

/// A `QueueItem` serialized with the previous `String` indexer field
/// must continue to deserialize correctly after the migration to [`Indexer`].
#[test]
fn indexer_deserialize__queue_item_with_legacy_string_indexer() {
    let yaml = "
name: Artist - Album (2018) [FLAC]
path: /downloads/example.torrent
hash: '0102030405060708090a0b0c0d0e0f1011121314'
indexer: red
id: 12345
";
    let item: QueueItem = serde_yaml::from_str(yaml).expect("should deserialize");
    assert_eq!(item.name, "Artist - Album (2018) [FLAC]");
    assert_eq!(item.indexer, Some(Indexer::Red));
    assert_eq!(item.id, Some(12345));
}

#[test]
fn indexer_deserialize__queue_item_with_legacy_unknown_indexer() {
    let yaml = "
name: Test Album
path: /downloads/test.torrent
hash: '0102030405060708090a0b0c0d0e0f1011121314'
indexer: abc
";
    let item: QueueItem = serde_yaml::from_str(yaml).expect("should deserialize");
    assert_eq!(item.indexer, Some(Indexer::Other("abc".to_owned())));
}

/// Legacy queue files written before the field became optional may contain
/// `indexer: ''` for items where the indexer could not be determined. The
/// empty string round-trips through [`Indexer::from`] to
/// [`Indexer::Other(String::new())`], which is functionally equivalent to
/// [`None`] for filtering since it never matches a known indexer.
#[test]
fn indexer_deserialize__queue_item_with_legacy_empty_indexer() {
    let yaml = "
name: Test Album
path: /downloads/test.torrent
hash: '0102030405060708090a0b0c0d0e0f1011121314'
indexer: ''
";
    let item: QueueItem = serde_yaml::from_str(yaml).expect("should deserialize");
    assert_eq!(item.indexer, Some(Indexer::Other(String::new())));
}

/// A queue file with no `indexer` field at all must deserialize to [`None`]
/// without error.
#[test]
fn indexer_deserialize__queue_item_with_missing_indexer() {
    let yaml = "
name: Test Album
path: /downloads/test.torrent
hash: '0102030405060708090a0b0c0d0e0f1011121314'
";
    let item: QueueItem = serde_yaml::from_str(yaml).expect("should deserialize");
    assert_eq!(item.indexer, None);
}

#[test]
fn indexer_match_with_alts() {
    assert!(test("ops", "ops"));
    assert!(test("pth", "pth"));
    assert!(test("red", "red"));
    assert!(test("red", "RED"));
    assert!(test("RED", "RED"));
    assert!(test("red", "pth"));
    assert!(test("red", "PTH"));
    assert!(test("abc", "AbC"));
    assert!(!test("red", "ops"));
    assert!(!test("red", "OPS"));
    assert!(!test("RED", "OPS"));
    assert!(!test("pth", "red"));
}

fn test(left: &str, right: &str) -> bool {
    let left = Indexer::from(left);
    let right = Indexer::from(right);
    left.match_with_alts(&right)
}
