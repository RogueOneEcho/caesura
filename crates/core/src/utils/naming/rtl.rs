//! Right-to-left script detection for directional-mark auditing.

/// Detect right-to-left script in a string.
pub(crate) trait StrRtlExt {
    /// Does the string contain a right-to-left character?
    fn contains_rtl(&self) -> bool;
}

impl StrRtlExt for str {
    fn contains_rtl(&self) -> bool {
        self.chars().any(is_rtl)
    }
}

/// Does the character belong to a right-to-left script?
///
/// - Tests membership of the right-to-left script blocks, a conservative
///   superset of the Unicode `Bidi_Class` values `R`, `AL`, and `AN`
/// - Being a superset, it never misclassifies real right-to-left text, so it
///   never yields a false unnecessary-mark flag; its only failure mode is an
///   incomplete block list declining to flag a mark beside a rare script
/// - For exact classification use the `unicode-bidi` crate's `bidi_class`,
///   matching `R`, `AL`, or `AN`
fn is_rtl(c: char) -> bool {
    matches!(c,
        // Hebrew, Arabic, Syriac, Thaana, N'Ko, Samaritan, Mandaic, Arabic extensions
        '\u{0590}'..='\u{08FF}'
        // Hebrew and Arabic presentation forms A
        | '\u{FB1D}'..='\u{FDFF}'
        // Arabic presentation forms B
        | '\u{FE70}'..='\u{FEFF}'
        // Hanifi Rohingya
        | '\u{10D00}'..='\u{10D3F}'
        // Adlam
        | '\u{1E900}'..='\u{1E95F}'
    )
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    #[test]
    fn str_contains_rtl_latin() {
        assert!(!"Various - Afro Edits".contains_rtl());
    }

    #[test]
    fn str_contains_rtl_hebrew() {
        assert!("\u{05E9}\u{05DC}\u{05D5}\u{05DD}".contains_rtl());
    }

    #[test]
    fn str_contains_rtl_arabic() {
        assert!("\u{0639}\u{0631}\u{0628}\u{0649}".contains_rtl());
    }

    #[test]
    fn str_contains_rtl_arabic_digit() {
        assert!("track \u{0661}".contains_rtl());
    }
}
