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

/// Path to the eyeD3 binary.
#[cfg(target_os = "windows")]
pub const EYED3: &str = "eyeD3.exe";

/// Path to the eyeD3 binary.
#[cfg(not(target_os = "windows"))]
pub const EYED3: &str = "eyeD3";

/// Path to the metaflac binary.
#[cfg(target_os = "windows")]
pub const METAFLAC: &str = "metaflac.exe";

/// Path to the metaflac binary.
#[cfg(not(target_os = "windows"))]
pub const METAFLAC: &str = "metaflac";
