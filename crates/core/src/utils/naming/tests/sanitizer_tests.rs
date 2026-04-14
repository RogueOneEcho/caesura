use crate::testing_prelude::*;

#[test]
fn sanitizer_name_restricted_chars() {
    // Arrange
    let input = format!(
        "Artist - Album ze{}ro () [2009]",
        SanitizerChar::ZeroWidthNoBreakSpace.get_char()
    );

    // Act
    let result = Sanitizer::name().execute(input);

    // Assert
    assert_eq!(result, "Artist - Album zero () [2009]");
}

#[test]
fn sanitizer_name_dividers() {
    // Arrange
    let input = "Artist - Album ze-ro () [2009]".to_owned();

    // Act
    let result = Sanitizer::name().execute(input);

    // Assert
    assert_eq!(result, "Artist - Album ze-ro () [2009]");
}

#[test]
fn sanitizer_name_en_dash() {
    // Arrange
    let input = format!(
        "Artist {} Album zero () [2009]",
        SanitizerChar::EnDash.get_char()
    );

    // Act
    let result = Sanitizer::name().execute(input);

    // Assert
    assert_eq!(result, "Artist - Album zero () [2009]");
}

#[test]
fn sanitizer_name_valid_unicode() {
    // Arrange
    let input = "안녕하세요 세상 - 你好世界 - こんにちは世界".to_owned();

    // Act
    let result = Sanitizer::name().execute(input.clone());

    // Assert
    assert_eq!(result, input);
}

#[test]
fn sanitizer_name_valid_emoji() {
    // Arrange
    let input = "⚡ 💻 🧠 👨‍💻 👨 💊 ☝️ 🛜 ".to_owned();

    // Act
    let result = Sanitizer::name().execute(input.clone());

    // Assert
    assert_eq!(result, input);
}

#[test]
fn sanitizer_libtorrent_no_change() {
    assert_eq!(
        Sanitizer::libtorrent().execute("no change".to_owned()),
        "no change"
    );
}

#[test]
fn sanitizer_libtorrent_strips_slashes() {
    let sanitizer = Sanitizer::libtorrent();
    assert_eq!(sanitizer.execute("AC/DC".to_owned()), "ACDC");
    assert_eq!(sanitizer.execute(r"back\slash".to_owned()), "backslash");
}

#[test]
fn sanitizer_libtorrent_strips_directional_marks() {
    let sanitizer = Sanitizer::libtorrent();
    assert_eq!(sanitizer.execute("a\u{200e}b\u{200f}c".to_owned()), "abc");
    assert_eq!(
        sanitizer.execute("Hello\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}World".to_owned()),
        "HelloWorld"
    );
}

#[test]
fn sanitizer_libtorrent_empty() {
    assert_eq!(Sanitizer::libtorrent().execute(String::new()), "");
}
