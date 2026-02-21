use crate::testing_prelude::*;

#[test]
fn remove_restricted_chars() {
    // Arrange
    let input = format!("Artist - Album ze{ZERO_WIDTH_NO_BREAK_SPACE}ro () [2009]");

    // Act
    let result = Sanitizer::execute(input);

    // Assert
    assert_eq!(result, "Artist - Album zero () [2009]");
}

#[test]
fn test_contains_dividers() {
    // Arrange
    let input = "Artist - Album ze-ro () [2009]".to_owned();

    // Act
    let result = Sanitizer::execute(input);

    // Assert
    assert_eq!(result, "Artist - Album ze-ro () [2009]");
}

#[test]
fn test_contains_en_dash() {
    // Arrange
    let input = format!("Artist {EN_DASH} Album zero () [2009]");

    // Act
    let result = Sanitizer::execute(input);

    // Assert
    assert_eq!(result, "Artist - Album zero () [2009]");
}

#[test]
fn test_contains_valid_unicode() {
    // Arrange
    let input = "안녕하세요 세상 - 你好世界 - こんにちは世界".to_owned();

    // Act
    let result = Sanitizer::execute(input.clone());

    // Assert
    assert_eq!(result, input);
}

#[test]
fn test_contains_valid_emoji() {
    // Arrange
    let input = "⚡ 💻 🧠 👨‍💻 👨 💊 ☝️ 🛜 ".to_owned();

    // Act
    let result = Sanitizer::execute(input.clone());

    // Assert
    assert_eq!(result, input);
}

#[test]
fn execute_strips_dots() {
    // Arrange
    let input = "Artist - Album.torrent".to_owned();

    // Act
    let result = Sanitizer::execute(input);

    // Assert
    assert_eq!(result, "Artist - Albumtorrent");
}

#[test]
fn validate_returns_false_on_restricted_chars() {
    assert!(!Sanitizer::validate("Artist: Album"));
    assert!(!Sanitizer::validate("folder.name"));
    assert!(!Sanitizer::validate("path/name"));
}

#[test]
fn validate_returns_true_on_clean_input() {
    assert!(Sanitizer::validate("Artist - Album [2020]"));
    assert!(Sanitizer::validate("My Custom Name"));
    assert!(Sanitizer::validate("안녕하세요"));
}
