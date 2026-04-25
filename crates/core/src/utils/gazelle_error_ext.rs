use crate::prelude::*;

/// Extension methods on [`GazelleError`].
pub(crate) trait GazelleErrorExt {
    /// Whether the error indicates the requested resource is missing.
    ///
    /// - True for `NotFound` and `BadRequest` API responses
    /// - The tracker returns `BadRequest` for some lookups against unknown ids
    fn is_missing(&self) -> bool;
}

impl GazelleErrorExt for GazelleError {
    fn is_missing(&self) -> bool {
        matches!(
            self.operation,
            GazelleOperation::ApiResponse(ApiResponseKind::NotFound | ApiResponseKind::BadRequest)
        )
    }
}
