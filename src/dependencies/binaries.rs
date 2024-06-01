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

// TODO MUST confirm binary name on windows
/// Path to the imagemagick convert binary.
#[cfg(target_os = "windows")]
pub const CONVERT: &str = "convert.exe";

/// Path to the imagemagick convert binary.
#[cfg(not(target_os = "windows"))]
pub const CONVERT: &str = "convert";

/// Path to the imdl binary.
#[cfg(target_os = "windows")]
pub const IMDL: &str = "imdl.exe";

/// Path to the imdl binary.
#[cfg(not(target_os = "windows"))]
pub const IMDL: &str = "imdl";
