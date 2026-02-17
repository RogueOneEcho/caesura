/// Path to the SoX-ng binary.
#[cfg(target_os = "windows")]
pub const SOX_NG: &str = "sox_ng.exe";

/// Path to the SoX-ng binary.
#[cfg(not(target_os = "windows"))]
pub const SOX_NG: &str = "sox_ng";

/// Path to the original sox binary.
#[cfg(target_os = "windows")]
pub const SOX: &str = "sox.exe";

/// Path to the original sox binary.
#[cfg(not(target_os = "windows"))]
pub const SOX: &str = "sox";

/// Path to the lame binary.
#[cfg(target_os = "windows")]
pub const LAME: &str = "lame.exe";

/// Path to the lame binary.
#[cfg(not(target_os = "windows"))]
pub const LAME: &str = "lame";

/// Path to the flac binary.
#[cfg(target_os = "windows")]
pub const FLAC: &str = "flac.exe";

/// Path to the flac binary.
#[cfg(not(target_os = "windows"))]
pub const FLAC: &str = "flac";

/// Path to the metaflac binary.
///
/// Only used in tests and demo mode for setting tags and embedding cover art in generated FLAC samples.
#[cfg(all(any(test, feature = "demo"), target_os = "windows"))]
pub const METAFLAC: &str = "metaflac.exe";

/// Path to the metaflac binary.
///
/// Only used in tests and demo mode for setting tags and embedding cover art in generated FLAC samples.
#[cfg(all(any(test, feature = "demo"), not(target_os = "windows")))]
pub const METAFLAC: &str = "metaflac";
