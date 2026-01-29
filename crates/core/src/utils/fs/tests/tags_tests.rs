use super::super::*;

#[test]
fn valid_total_formats() {
    assert_eq!(get_numeric_from_total_format("1/1"), Some((1, 1)));
    assert_eq!(get_numeric_from_total_format("1/12"), Some((1, 12)));
    assert_eq!(get_numeric_from_total_format("2/6"), Some((2, 6)));
    assert_eq!(get_numeric_from_total_format("3/8"), Some((3, 8)));
    assert_eq!(get_numeric_from_total_format("26/9"), Some((26, 9)));
}

#[test]
fn invalid_total_formats() {
    assert_eq!(get_numeric_from_total_format(""), None);
    assert_eq!(get_numeric_from_total_format("1"), None);
    assert_eq!(get_numeric_from_total_format("12"), None);
    assert_eq!(get_numeric_from_total_format("0"), None);
}

#[test]
fn valid_vinyl_formats() {
    assert_eq!(get_numeric_from_vinyl_format("A1"), Some((1, 1)));
    assert_eq!(get_numeric_from_vinyl_format("A12"), Some((1, 12)));
    assert_eq!(get_numeric_from_vinyl_format("B6"), Some((2, 6)));
    assert_eq!(get_numeric_from_vinyl_format("C8"), Some((3, 8)));
    assert_eq!(get_numeric_from_vinyl_format("Z9"), Some((26, 9)));
}

#[test]
fn invalid_vinyl_formats() {
    assert_eq!(get_numeric_from_vinyl_format(""), None);
    assert_eq!(get_numeric_from_vinyl_format("A"), None);
    assert_eq!(get_numeric_from_vinyl_format("12"), None);
    assert_eq!(get_numeric_from_vinyl_format("1A"), None);
}
