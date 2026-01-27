/// Audio bit depth.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Depth {
    /// 16-bit depth.
    _16,
    /// 24-bit depth.
    _24,
}

impl Depth {
    /// Return the numeric bit depth.
    pub const fn as_u16(self) -> u16 {
        match self {
            Self::_16 => 16,
            Self::_24 => 24,
        }
    }
}

/// Audio sample rate.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rate {
    /// 44.1 kHz (CD quality).
    _44100,
    /// 48 kHz.
    _48000,
    /// 96 kHz.
    _96000,
    /// 192 kHz.
    _192000,
}

impl Rate {
    /// Return the numeric sample rate in Hz.
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::_44100 => 44100,
            Self::_48000 => 48000,
            Self::_96000 => 96000,
            Self::_192000 => 192_000,
        }
    }

    pub(super) fn dir_suffix(self) -> &'static str {
        match self {
            Self::_44100 => "44.1",
            Self::_48000 => "48",
            Self::_96000 => "96",
            Self::_192000 => "192",
        }
    }
}

/// Audio format specification combining bit depth and sample rate.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleFormat {
    /// Bit depth.
    pub depth: Depth,
    /// Sample rate.
    pub rate: Rate,
}

impl SampleFormat {
    /// Standard CD quality: 16-bit, 44.1 kHz.
    pub const FLAC16_441: Self = Self {
        depth: Depth::_16,
        rate: Rate::_44100,
    };

    /// 16-bit, 48 kHz.
    pub const FLAC16_48: Self = Self {
        depth: Depth::_16,
        rate: Rate::_48000,
    };

    /// 24-bit, 44.1 kHz.
    pub const FLAC24_441: Self = Self {
        depth: Depth::_24,
        rate: Rate::_44100,
    };

    /// 24-bit, 48 kHz.
    pub const FLAC24_48: Self = Self {
        depth: Depth::_24,
        rate: Rate::_48000,
    };

    /// 24-bit, 96 kHz.
    pub const FLAC24_96: Self = Self {
        depth: Depth::_24,
        rate: Rate::_96000,
    };

    pub(super) fn dir_suffix(self) -> String {
        format!("{{{}-{}}}", self.depth.as_u16(), self.rate.dir_suffix())
    }
}
