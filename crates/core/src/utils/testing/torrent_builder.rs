use crate::testing_prelude::*;

const MOCK_FILE_SIZE: i64 = 1024;

const MOCK_NAME: &str = "album";

/// Builds bencode-encoded test torrents, preserving entry order.
pub(crate) struct TorrentBuilder {
    /// Encoded dictionary entries in insertion order.
    entries: Vec<u8>,
}

impl TorrentBuilder {
    /// Create an empty [`TorrentBuilder`].
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create a file [`TorrentBuilder`] with a fixed length and `path` of `components`.
    pub fn file<I>(components: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<RawString>,
    {
        Self::new()
            .with_integer("length", MOCK_FILE_SIZE)
            .with_list("path", components)
    }

    /// Append a byte-string entry encoded as `<len>:<bytes>`.
    pub fn with_string(mut self, key: &str, value: impl Into<RawString>) -> Self {
        self.push_bytes(key.as_bytes());
        self.push_bytes(value.into().as_bytes());
        self
    }

    /// Append an integer entry encoded as `i<value>e`.
    pub fn with_integer(mut self, key: &str, value: i64) -> Self {
        self.push_bytes(key.as_bytes());
        self.entries
            .extend_from_slice(format!("i{value}e").as_bytes());
        self
    }

    /// Append a list of byte-string entries encoded as `l<items>e`.
    pub fn with_list<I>(mut self, key: &str, items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<RawString>,
    {
        self.push_bytes(key.as_bytes());
        self.entries.push(b'l');
        for item in items {
            self.push_bytes(item.into().as_bytes());
        }
        self.entries.push(b'e');
        self
    }

    /// Append a nested dictionary entry.
    pub fn with_dictionary(mut self, key: &str, dictionary: TorrentBuilder) -> Self {
        self.push_bytes(key.as_bytes());
        self.entries.extend_from_slice(&dictionary.build());
        self
    }

    /// Append a list of dictionaries encoded as `l<dictionaries>e`.
    pub fn with_dictionaries(mut self, key: &str, dictionaries: Vec<TorrentBuilder>) -> Self {
        self.push_bytes(key.as_bytes());
        self.entries.push(b'l');
        for dictionary in dictionaries {
            self.entries.extend_from_slice(&dictionary.build());
        }
        self.entries.push(b'e');
        self
    }

    /// Append an `info` dictionary for a multi-file torrent named `album`.
    ///
    /// - Adds one file with a fixed length and `path` of `components`
    pub fn with_multi_file<I>(self, components: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<RawString>,
    {
        self.with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(components)])
                .with_string("name", MOCK_NAME),
        )
    }

    /// Encode the accumulated entries as a bencode dictionary (`d…e`).
    pub fn build(self) -> Vec<u8> {
        let mut bytes = vec![b'd'];
        bytes.extend_from_slice(&self.entries);
        bytes.push(b'e');
        bytes
    }

    /// Append a bencode byte string (`<len>:<bytes>`).
    fn push_bytes(&mut self, bytes: &[u8]) {
        self.entries
            .extend_from_slice(format!("{}:", bytes.len()).as_bytes());
        self.entries.extend_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    /// A nested structure encodes to the expected bencode byte string.
    #[test]
    fn torrent_builder_build() {
        // Arrange
        let builder = TorrentBuilder::new().with_dictionary(
            "info",
            TorrentBuilder::new()
                .with_dictionaries("files", vec![TorrentBuilder::file(["song.flac"])])
                .with_string("name", "album"),
        );

        // Act
        let output = builder.build();

        // Assert
        assert_eq!(
            output,
            b"d4:infod5:filesld6:lengthi1024e4:pathl9:song.flaceee4:name5:albumee".to_vec()
        );
    }
}
