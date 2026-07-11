use crate::prelude::*;

/// A path or name component after decoding, or its candidate decodings.
pub enum DecodedString {
    /// A component decoded to a single known UTF-8 value.
    Known(String),
    /// Raw bytes that could not be decoded unambiguously, with candidate decodings.
    Suggestions(Vec<u8>, Vec<AuditSuggestion>),
}

impl Display for DecodedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DecodedString::Known(value) => write!(f, "{value}"),
            DecodedString::Suggestions(raw, suggestions) => {
                if let Some(suggestion) = suggestions.first() {
                    write!(f, "{}", suggestion.value)
                } else {
                    write!(f, "{}", String::from_utf8_lossy(raw))
                }
            }
        }
    }
}
