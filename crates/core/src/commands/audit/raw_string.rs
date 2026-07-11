use crate::prelude::*;
use lava_torrent::bencode::BencodeElem;
use std::string::FromUtf8Error;

/// A bencode string value, either valid UTF-8 or raw bytes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum RawString {
    /// A valid UTF-8 string.
    String(String),
    /// Raw bytes that were not decoded as UTF-8.
    Bytes(Vec<u8>),
}

impl RawString {
    /// Decode to UTF-8, replacing undecodable bytes with `U+FFFD`.
    pub fn to_string_lossy(&self) -> String {
        match self {
            RawString::String(string) => string.clone(),
            RawString::Bytes(bytes) => String::from_utf8_lossy(bytes).to_string(),
        }
    }

    /// Whether the value has no characters.
    pub fn is_empty(&self) -> bool {
        match self {
            RawString::String(value) => value.is_empty(),
            RawString::Bytes(bytes) => bytes.is_empty(),
        }
    }

    /// Decode to UTF-8, returning an error for invalid bytes.
    ///
    /// # Errors
    ///
    /// Returns [`FromUtf8Error`] when the bytes are not valid UTF-8.
    pub fn try_to_string(&self) -> Result<String, FromUtf8Error> {
        match self {
            RawString::String(value) => Ok(value.clone()),
            RawString::Bytes(bytes) => String::from_utf8(bytes.clone()),
        }
    }

    /// Decode to UTF-8, rendering each invalid byte as its hex value.
    ///
    /// - Passes valid UTF-8 runs through verbatim
    /// - Renders each invalid byte as `<` + uppercase hex + `>`, e.g. `<E9>`
    pub fn to_string_with_hex(&self) -> String {
        let mut output = String::new();
        for chunk in self.as_bytes().utf8_chunks() {
            output.push_str(chunk.valid());
            for byte in chunk.invalid() {
                write!(output, "<{byte:02X}>").expect("should write");
            }
        }
        output
    }

    /// Borrow the underlying bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            RawString::String(value) => value.as_bytes(),
            RawString::Bytes(bytes) => bytes,
        }
    }
}

impl Display for RawString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl TryFrom<BencodeElem> for RawString {
    type Error = AuditIssueKind;

    fn try_from(bencode: BencodeElem) -> Result<Self, Self::Error> {
        match bencode {
            BencodeElem::String(value) => Ok(Self::String(value)),
            BencodeElem::Bytes(bytes) => Ok(Self::Bytes(bytes)),
            _ => Err(AuditIssueKind::NotStringOrBytes),
        }
    }
}

impl From<&str> for RawString {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for RawString {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for RawString {
    fn from(value: &String) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<&[u8]> for RawString {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(value.to_vec())
    }
}

impl From<Vec<u8>> for RawString {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}
