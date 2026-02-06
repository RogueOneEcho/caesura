use crate::prelude::*;

/// Actions that can fail in the spectrogram module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
pub enum SpectrogramAction {
    #[error("get source from options")]
    GetSource,
    #[error("create output directory")]
    CreateOutputDirectory,
    #[error("generate spectrogram")]
    GenerateSpectrogram,
    #[error("execute spectrogram runner")]
    ExecuteRunner,
}
