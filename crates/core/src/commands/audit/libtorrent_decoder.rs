const MAX_CODEPOINT: i32 = 0x0010_FFFF;
const SURROGATE_START: i32 = 0x0000_D800;
const SURROGATE_END: i32 = 0x0000_DFFF;

/// Predict the on-disk name libtorrent writes for a raw path element.
///
/// Ports the UTF-8 handling of libtorrent's `sanitize_append_path_element`.
/// - <https://github.com/arvidn/libtorrent/blob/RC_2_0/src/torrent_info.cpp>
/// - <https://github.com/arvidn/libtorrent/blob/RC_2_0/src/utf8.cpp>
///
/// # Caution
///
/// This recreation of the libtorrent logic was written by Claude Code Opus 4.8.
///
/// - Inputs and outputs have been validated
/// - Core logic may not be an exact match
pub(crate) struct LibtorrentDecoder;

impl LibtorrentDecoder {
    /// Decode raw path `bytes` the way libtorrent writes them to disk.
    ///
    /// - Copies each valid UTF-8 sequence through unchanged
    /// - Replaces each invalid sequence with a single `_`, advancing by the
    ///   lead byte's claimed length so a trailing `.` can be consumed
    pub(crate) fn decode(bytes: &[u8]) -> String {
        let mut output = String::new();
        let mut remaining = bytes;
        while !remaining.is_empty() {
            let (code_point, length) = Self::parse_codepoint(remaining);
            if code_point < 0 {
                output.push('_');
            } else if let Some(sequence) = remaining
                .get(..length)
                .and_then(|sequence| str::from_utf8(sequence).ok())
            {
                output.push_str(sequence);
            }
            remaining = remaining.get(length..).unwrap_or_default();
        }
        output
    }

    /// Parse the leading UTF-8 code point of `bytes`.
    ///
    /// - Returns the code point and the number of bytes it occupies
    /// - Returns a negative code point for an invalid sequence, paired with the
    ///   number of bytes to skip
    fn parse_codepoint(bytes: &[u8]) -> (i32, usize) {
        let Some(&first) = bytes.first() else {
            return (-1, 1);
        };
        let length = Self::sequence_length(first);
        if length == 1 {
            return (i32::from(first), 1);
        }
        if length == 0 {
            return (-1, 1);
        }
        if length > 4 {
            return (-1, length);
        }
        let Some(continuation) = bytes.get(1..length) else {
            return (-1, bytes.len());
        };
        let mut code_point = match length {
            2 => i32::from(first & 0b0001_1111),
            3 => i32::from(first & 0b0000_1111),
            _ => i32::from(first & 0b0000_0111),
        };
        for &byte in continuation {
            if !(0b1000_0000..=0b1011_1111).contains(&byte) {
                return (-1, length);
            }
            code_point = (code_point << 6) + i32::from(byte & 0b0011_1111);
        }
        let overlong = match length {
            2 => code_point < 0x0000_0080,
            3 => code_point < 0x0000_0800,
            _ => code_point < 0x0001_0000,
        };
        if overlong || code_point > MAX_CODEPOINT {
            return (-1, length);
        }
        if (SURROGATE_START..=SURROGATE_END).contains(&code_point) {
            return (-1, length);
        }
        (code_point, length)
    }

    /// Number of bytes claimed by a UTF-8 lead byte, or `0` when invalid.
    fn sequence_length(byte: u8) -> usize {
        if byte < 0b1000_0000 {
            return 1;
        }
        if byte >> 5 == 0b110 {
            return 2;
        }
        if byte >> 4 == 0b1110 {
            return 3;
        }
        if byte >> 3 == 0b0001_1110 {
            return 4;
        }
        if byte >> 2 == 0b0011_1110 {
            return 5;
        }
        0
    }
}
