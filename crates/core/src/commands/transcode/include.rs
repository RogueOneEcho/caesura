use crate::prelude::*;

/// Information needed to copy a FLAC that does not need re-sampling.
pub(crate) struct Include {
    /// Path to the input file
    pub input: PathBuf,
    /// Path to the output file
    pub output: PathBuf,
    /// Should the file be hard linked?
    pub hard_link: bool,
}
