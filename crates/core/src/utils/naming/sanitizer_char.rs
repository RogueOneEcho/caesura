//! Character constants and classification for path sanitization.

use crate::prelude::*;

/// Characters that are illegal or problematic in file paths.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ThisError)]
pub(crate) enum SanitizerChar {
    // Filesystem illegal
    #[error("colon")]
    Colon,
    #[error("less than")]
    LessThan,
    #[error("greater than")]
    GreaterThan,
    #[error("double quote")]
    DoubleQuote,
    #[error("question mark")]
    QuestionMark,
    #[error("asterisk")]
    Asterisk,
    // Dividers (replaced with hyphen)
    #[error("forward slash")]
    ForwardSlash,
    #[error("backslash")]
    Backslash,
    #[error("pipe")]
    Pipe,
    #[error("en dash")]
    EnDash,
    #[error("em dash")]
    EmDash,
    // Unicode whitespace/formatting
    #[error("non-breaking space")]
    NonBreakingSpace,
    #[error("zero-width space")]
    ZeroWidthSpace,
    #[error("left-to-right mark")]
    LeftToRightMark,
    #[error("right-to-left mark")]
    RightToLeftMark,
    #[error("left-to-right embedding")]
    LeftToRightEmbedding,
    #[error("right-to-left embedding")]
    RightToLeftEmbedding,
    #[error("pop directional formatting")]
    PopDirectionalFormatting,
    #[error("left-to-right override")]
    LeftToRightOverride,
    #[error("right-to-left override")]
    RightToLeftOverride,
    #[error("zero-width no-break space")]
    ZeroWidthNoBreakSpace,
    // C0 control characters
    #[error("null")]
    Null,
    #[error("start of heading")]
    StartOfHeading,
    #[error("start of text")]
    StartOfText,
    #[error("end of text")]
    EndOfText,
    #[error("end of transmission")]
    EndOfTransmission,
    #[error("enquiry")]
    Enquiry,
    #[error("acknowledge")]
    Acknowledge,
    #[error("bell")]
    Bell,
    #[error("backspace")]
    Backspace,
    #[error("horizontal tab")]
    HorizontalTab,
    #[error("line feed")]
    LineFeed,
    #[error("vertical tab")]
    VerticalTab,
    #[error("form feed")]
    FormFeed,
    #[error("carriage return")]
    CarriageReturn,
    #[error("shift out")]
    ShiftOut,
    #[error("shift in")]
    ShiftIn,
    #[error("data link escape")]
    DataLinkEscape,
    #[error("device control 1")]
    DeviceControl1,
    #[error("device control 2")]
    DeviceControl2,
    #[error("device control 3")]
    DeviceControl3,
    #[error("device control 4")]
    DeviceControl4,
    #[error("negative acknowledge")]
    NegativeAcknowledge,
    #[error("synchronous idle")]
    SynchronousIdle,
    #[error("end of transmission block")]
    EndOfTransmissionBlock,
    #[error("cancel")]
    Cancel,
    #[error("end of medium")]
    EndOfMedium,
    #[error("substitute")]
    Substitute,
    #[error("escape")]
    Escape,
    #[error("file separator")]
    FileSeparator,
    #[error("group separator")]
    GroupSeparator,
    #[error("record separator")]
    RecordSeparator,
    #[error("unit separator")]
    UnitSeparator,
    #[error("delete")]
    Delete,
    // C1 control characters
    #[error("padding character")]
    PaddingCharacter,
    #[error("high octet preset")]
    HighOctetPreset,
    #[error("break permitted here")]
    BreakPermittedHere,
    #[error("no break here")]
    NoBreakHere,
    #[error("index")]
    Index,
    #[error("next line")]
    NextLine,
    #[error("start of selected area")]
    StartOfSelectedArea,
    #[error("end of selected area")]
    EndOfSelectedArea,
    #[error("character tabulation set")]
    CharacterTabulationSet,
    #[error("character tabulation with justification")]
    CharacterTabulationWithJustification,
    #[error("line tabulation set")]
    LineTabulationSet,
    #[error("partial line forward")]
    PartialLineForward,
    #[error("partial line backward")]
    PartialLineBackward,
    #[error("reverse line feed")]
    ReverseLineFeed,
    #[error("single shift two")]
    SingleShiftTwo,
    #[error("single shift three")]
    SingleShiftThree,
    #[error("device control string")]
    DeviceControlString,
    #[error("private use one")]
    PrivateUseOne,
    #[error("private use two")]
    PrivateUseTwo,
    #[error("set transmit state")]
    SetTransmitState,
    #[error("cancel character")]
    CancelCharacter,
    #[error("message waiting")]
    MessageWaiting,
    #[error("start of guarded area")]
    StartOfGuardedArea,
    #[error("end of guarded area")]
    EndOfGuardedArea,
    #[error("start of string")]
    StartOfString,
    #[error("single graphic character introducer")]
    SingleGraphicCharacterIntroducer,
    #[error("single character introducer")]
    SingleCharacterIntroducer,
    #[error("control sequence introducer")]
    ControlSequenceIntroducer,
    #[error("string terminator")]
    StringTerminator,
    #[error("operating system command")]
    OperatingSystemCommand,
    #[error("privacy message")]
    PrivacyMessage,
    #[error("application program command")]
    ApplicationProgramCommand,
}

impl SanitizerChar {
    /// The `char` this variant represents.
    #[must_use]
    pub fn get_char(self) -> char {
        match self {
            Self::Colon => ':',
            Self::LessThan => '<',
            Self::GreaterThan => '>',
            Self::DoubleQuote => '"',
            Self::QuestionMark => '?',
            Self::Asterisk => '*',
            Self::ForwardSlash => '/',
            Self::Backslash => '\\',
            Self::Pipe => '|',
            Self::EnDash => '\u{2013}',
            Self::EmDash => '\u{2014}',
            Self::NonBreakingSpace => '\u{00A0}',
            Self::ZeroWidthSpace => '\u{200B}',
            Self::LeftToRightMark => '\u{200E}',
            Self::RightToLeftMark => '\u{200F}',
            Self::LeftToRightEmbedding => '\u{202A}',
            Self::RightToLeftEmbedding => '\u{202B}',
            Self::PopDirectionalFormatting => '\u{202C}',
            Self::LeftToRightOverride => '\u{202D}',
            Self::RightToLeftOverride => '\u{202E}',
            Self::ZeroWidthNoBreakSpace => '\u{FEFF}',
            Self::Null => '\x00',
            Self::StartOfHeading => '\x01',
            Self::StartOfText => '\x02',
            Self::EndOfText => '\x03',
            Self::EndOfTransmission => '\x04',
            Self::Enquiry => '\x05',
            Self::Acknowledge => '\x06',
            Self::Bell => '\x07',
            Self::Backspace => '\x08',
            Self::HorizontalTab => '\x09',
            Self::LineFeed => '\x0A',
            Self::VerticalTab => '\x0B',
            Self::FormFeed => '\x0C',
            Self::CarriageReturn => '\x0D',
            Self::ShiftOut => '\x0E',
            Self::ShiftIn => '\x0F',
            Self::DataLinkEscape => '\x10',
            Self::DeviceControl1 => '\x11',
            Self::DeviceControl2 => '\x12',
            Self::DeviceControl3 => '\x13',
            Self::DeviceControl4 => '\x14',
            Self::NegativeAcknowledge => '\x15',
            Self::SynchronousIdle => '\x16',
            Self::EndOfTransmissionBlock => '\x17',
            Self::Cancel => '\x18',
            Self::EndOfMedium => '\x19',
            Self::Substitute => '\x1A',
            Self::Escape => '\x1B',
            Self::FileSeparator => '\x1C',
            Self::GroupSeparator => '\x1D',
            Self::RecordSeparator => '\x1E',
            Self::UnitSeparator => '\x1F',
            Self::Delete => '\x7F',
            Self::PaddingCharacter => '\u{0080}',
            Self::HighOctetPreset => '\u{0081}',
            Self::BreakPermittedHere => '\u{0082}',
            Self::NoBreakHere => '\u{0083}',
            Self::Index => '\u{0084}',
            Self::NextLine => '\u{0085}',
            Self::StartOfSelectedArea => '\u{0086}',
            Self::EndOfSelectedArea => '\u{0087}',
            Self::CharacterTabulationSet => '\u{0088}',
            Self::CharacterTabulationWithJustification => '\u{0089}',
            Self::LineTabulationSet => '\u{008A}',
            Self::PartialLineForward => '\u{008B}',
            Self::PartialLineBackward => '\u{008C}',
            Self::ReverseLineFeed => '\u{008D}',
            Self::SingleShiftTwo => '\u{008E}',
            Self::SingleShiftThree => '\u{008F}',
            Self::DeviceControlString => '\u{0090}',
            Self::PrivateUseOne => '\u{0091}',
            Self::PrivateUseTwo => '\u{0092}',
            Self::SetTransmitState => '\u{0093}',
            Self::CancelCharacter => '\u{0094}',
            Self::MessageWaiting => '\u{0095}',
            Self::StartOfGuardedArea => '\u{0096}',
            Self::EndOfGuardedArea => '\u{0097}',
            Self::StartOfString => '\u{0098}',
            Self::SingleGraphicCharacterIntroducer => '\u{0099}',
            Self::SingleCharacterIntroducer => '\u{009A}',
            Self::ControlSequenceIntroducer => '\u{009B}',
            Self::StringTerminator => '\u{009C}',
            Self::OperatingSystemCommand => '\u{009D}',
            Self::PrivacyMessage => '\u{009E}',
            Self::ApplicationProgramCommand => '\u{009F}',
        }
    }
}
