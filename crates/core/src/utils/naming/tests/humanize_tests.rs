use super::super::*;
use std::collections::BTreeSet;

#[test]
fn and_join_with_slice() {
    let strings = vec!["apple", "banana", "cherry"];
    let result = and_join(&strings);
    assert_eq!(result, "apple, banana & cherry");

    let strings = vec!["apple"];
    let result = and_join(&strings);
    assert_eq!(result, "apple");

    let strings: Vec<&str> = vec![];
    let result = and_join(&strings);
    assert_eq!(result, "");
}

#[test]
fn and_join_with_btreeset() {
    let set: BTreeSet<&str> = BTreeSet::from(["apple", "banana", "cherry"]);
    let result = and_join(&set);
    assert_eq!(result, "apple, banana & cherry");

    let set: BTreeSet<&str> = BTreeSet::from(["apple"]);
    let result = and_join(&set);
    assert_eq!(result, "apple");

    let set: BTreeSet<&str> = BTreeSet::new();
    let result = and_join(&set);
    assert_eq!(result, "");
}
