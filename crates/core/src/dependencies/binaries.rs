/// Path to the sox binary.
#[cfg(target_os = "windows")]
pub const SOX: &str = "sox.exe";

/// Path to the sox binary.
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

/// Path to the imdl binary.
#[cfg(target_os = "windows")]
pub const IMDL: &str = "imdl.exe";

/// Path to the imdl binary.
#[cfg(not(target_os = "windows"))]
pub const IMDL: &str = "imdl";

/// Path to the metaflac binary.
///
/// Only used in tests for setting tags and embedding cover art in generated FLAC samples.
#[cfg(all(test, target_os = "windows"))]
pub const METAFLAC: &str = "metaflac.exe";

/// Path to the metaflac binary.
///
/// Only used in tests for setting tags and embedding cover art in generated FLAC samples.
#[cfg(all(test, not(target_os = "windows")))]
pub const METAFLAC: &str = "metaflac";
