use crate::testing_prelude::*;

#[test]
fn join_humanized_with_slice() {
    let strings = vec!["apple", "banana", "cherry"];
    let result = join_humanized(&strings);
    assert_eq!(result, "apple, banana & cherry");

    let strings = vec!["apple"];
    let result = join_humanized(&strings);
    assert_eq!(result, "apple");

    let strings: Vec<&str> = vec![];
    let result = join_humanized(&strings);
    assert_eq!(result, "");
}

#[test]
fn join_humanized_with_btreeset() {
    let set: BTreeSet<&str> = BTreeSet::from(["apple", "banana", "cherry"]);
    let result = join_humanized(&set);
    assert_eq!(result, "apple, banana & cherry");

    let set: BTreeSet<&str> = BTreeSet::from(["apple"]);
    let result = join_humanized(&set);
    assert_eq!(result, "apple");

    let set: BTreeSet<&str> = BTreeSet::new();
    let result = join_humanized(&set);
    assert_eq!(result, "");
}
