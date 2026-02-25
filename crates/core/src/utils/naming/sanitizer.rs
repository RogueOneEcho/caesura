use crate::prelude::*;
use std::collections::HashSet;

/// Remove or replace characters that are invalid in file paths.
pub(crate) struct Sanitizer {
    rules: Vec<SanitizerRule>,
}

/// A rule that matches a set of characters and either removes or replaces them.
struct SanitizerRule {
    /// Characters matched by this rule.
    chars: Vec<SanitizerChar>,
    /// Replacement character, or `None` to remove.
    replacement: Option<char>,
}

/// Result of sanitizing a string, including the cleaned output and any matched characters.
#[derive(Debug)]
pub(crate) struct SanitizerResult {
    /// The sanitized string.
    pub output: String,
    /// Characters that were found and removed or replaced.
    pub found: HashSet<SanitizerChar>,
}

impl Sanitizer {
    /// Exclude invisible and control characters.
    #[must_use]
    pub fn invisible() -> Self {
        Self {
            rules: vec![SanitizerRule::invisible()],
        }
    }

    /// Sanitize source names.
    /// - Replace divider characters with hyphens
    /// - Remove restricted, invisible, and control characters
    #[must_use]
    pub fn name() -> Self {
        Self {
            rules: vec![
                SanitizerRule::replace_dividers(),
                SanitizerRule::restricted(),
            ],
        }
    }

    /// Sanitize with libtorrent rules.
    /// - <https://github.com/arvidn/libtorrent/blob/9c1897645265c6a450930e766ab46c02a240891f/src/torrent_info.cpp#L100>
    #[must_use]
    pub fn libtorrent() -> Self {
        Self {
            rules: vec![SanitizerRule::libtorrent()],
        }
    }

    /// Sanitize a string for use in file paths.
    #[must_use]
    pub fn execute(&self, input: String) -> SanitizerResult {
        let mut found = HashSet::new();
        let output = input
            .chars()
            .filter_map(|x| self.sanitize_char(x, &mut found))
            .collect();
        SanitizerResult { output, found }
    }

    fn sanitize_char(&self, char: char, found: &mut HashSet<SanitizerChar>) -> Option<char> {
        for rule in &self.rules {
            for rule_char in &rule.chars {
                if rule_char.get_char() == char {
                    found.insert(*rule_char);
                    return rule.replacement;
                }
            }
        }
        Some(char)
    }
}

impl SanitizerRule {
    /// Characters stripped by libtorrent from torrent file paths.
    ///
    /// - <https://github.com/arvidn/libtorrent/blob/9c1897645265c6a450930e766ab46c02a240891f/src/torrent_info.cpp#L100>
    fn libtorrent() -> Self {
        SanitizerRule {
            chars: vec![
                SanitizerChar::ForwardSlash,
                SanitizerChar::Backslash,
                SanitizerChar::LeftToRightMark,
                SanitizerChar::RightToLeftMark,
                SanitizerChar::LeftToRightEmbedding,
                SanitizerChar::RightToLeftEmbedding,
                SanitizerChar::PopDirectionalFormatting,
                SanitizerChar::LeftToRightOverride,
                SanitizerChar::RightToLeftOverride,
            ],
            replacement: None,
        }
    }

    /// Replace divider characters with a hyphen.
    fn replace_dividers() -> Self {
        SanitizerRule {
            chars: vec![
                SanitizerChar::ForwardSlash,
                SanitizerChar::Backslash,
                SanitizerChar::Pipe,
                SanitizerChar::EnDash,
                SanitizerChar::EmDash,
            ],
            replacement: Some('-'),
        }
    }

    /// Characters that should not be in file paths.
    ///
    /// Includes invisible and control characters.
    fn restricted() -> Self {
        Self::invisible().extend(vec![
            SanitizerChar::Colon,
            SanitizerChar::LessThan,
            SanitizerChar::GreaterThan,
            SanitizerChar::DoubleQuote,
            SanitizerChar::QuestionMark,
            SanitizerChar::Asterisk,
        ])
    }

    /// Invisible + control characters.
    fn invisible() -> Self {
        Self::control().extend(vec![
            SanitizerChar::NonBreakingSpace,
            SanitizerChar::ZeroWidthSpace,
            SanitizerChar::LeftToRightMark,
            SanitizerChar::RightToLeftMark,
            SanitizerChar::LeftToRightEmbedding,
            SanitizerChar::RightToLeftEmbedding,
            SanitizerChar::PopDirectionalFormatting,
            SanitizerChar::LeftToRightOverride,
            SanitizerChar::RightToLeftOverride,
            SanitizerChar::ZeroWidthNoBreakSpace,
        ])
    }

    /// C0 and C1 control characters.
    fn control() -> Self {
        SanitizerRule {
            chars: vec![
                SanitizerChar::Null,
                SanitizerChar::StartOfHeading,
                SanitizerChar::StartOfText,
                SanitizerChar::EndOfText,
                SanitizerChar::EndOfTransmission,
                SanitizerChar::Enquiry,
                SanitizerChar::Acknowledge,
                SanitizerChar::Bell,
                SanitizerChar::Backspace,
                SanitizerChar::HorizontalTab,
                SanitizerChar::LineFeed,
                SanitizerChar::VerticalTab,
                SanitizerChar::FormFeed,
                SanitizerChar::CarriageReturn,
                SanitizerChar::ShiftOut,
                SanitizerChar::ShiftIn,
                SanitizerChar::DataLinkEscape,
                SanitizerChar::DeviceControl1,
                SanitizerChar::DeviceControl2,
                SanitizerChar::DeviceControl3,
                SanitizerChar::DeviceControl4,
                SanitizerChar::NegativeAcknowledge,
                SanitizerChar::SynchronousIdle,
                SanitizerChar::EndOfTransmissionBlock,
                SanitizerChar::Cancel,
                SanitizerChar::EndOfMedium,
                SanitizerChar::Substitute,
                SanitizerChar::Escape,
                SanitizerChar::FileSeparator,
                SanitizerChar::GroupSeparator,
                SanitizerChar::RecordSeparator,
                SanitizerChar::UnitSeparator,
                SanitizerChar::Delete,
                SanitizerChar::PaddingCharacter,
                SanitizerChar::HighOctetPreset,
                SanitizerChar::BreakPermittedHere,
                SanitizerChar::NoBreakHere,
                SanitizerChar::Index,
                SanitizerChar::NextLine,
                SanitizerChar::StartOfSelectedArea,
                SanitizerChar::EndOfSelectedArea,
                SanitizerChar::CharacterTabulationSet,
                SanitizerChar::CharacterTabulationWithJustification,
                SanitizerChar::LineTabulationSet,
                SanitizerChar::PartialLineForward,
                SanitizerChar::PartialLineBackward,
                SanitizerChar::ReverseLineFeed,
                SanitizerChar::SingleShiftTwo,
                SanitizerChar::SingleShiftThree,
                SanitizerChar::DeviceControlString,
                SanitizerChar::PrivateUseOne,
                SanitizerChar::PrivateUseTwo,
                SanitizerChar::SetTransmitState,
                SanitizerChar::CancelCharacter,
                SanitizerChar::MessageWaiting,
                SanitizerChar::StartOfGuardedArea,
                SanitizerChar::EndOfGuardedArea,
                SanitizerChar::StartOfString,
                SanitizerChar::SingleGraphicCharacterIntroducer,
                SanitizerChar::SingleCharacterIntroducer,
                SanitizerChar::ControlSequenceIntroducer,
                SanitizerChar::StringTerminator,
                SanitizerChar::OperatingSystemCommand,
                SanitizerChar::PrivacyMessage,
                SanitizerChar::ApplicationProgramCommand,
            ],
            replacement: None,
        }
    }

    fn extend(mut self, vec: Vec<SanitizerChar>) -> Self {
        self.chars.extend(vec);
        self
    }
}

impl SanitizerResult {
    /// Human-readable summary of the characters that were found.
    pub fn humanize(&self) -> String {
        let found = self.found.iter().collect::<Vec<_>>();
        join_humanized(found)
    }
}

impl From<SanitizerResult> for String {
    fn from(result: SanitizerResult) -> Self {
        result.output
    }
}

impl AsRef<str> for SanitizerResult {
    fn as_ref(&self) -> &str {
        &self.output
    }
}

impl PartialEq<&str> for SanitizerResult {
    fn eq(&self, other: &&str) -> bool {
        self.output == *other
    }
}

impl PartialEq<String> for SanitizerResult {
    fn eq(&self, other: &String) -> bool {
        self.output == *other
    }
}
