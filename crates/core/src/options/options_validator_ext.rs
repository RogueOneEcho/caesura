//! Extension trait bridging [`OptionsValidator`] to [`Failure`].

use crate::prelude::*;
use rogue_logging::Action;

/// Extension methods on [`OptionsValidator`] that depend on [`Failure`].
pub trait OptionsValidatorExt {
    /// Check for issues, logging any that were collected.
    ///
    /// - Returns `Ok(())` if no issues were collected.
    /// - Returns [`Failure`] from `action` if any issues were collected.
    fn check_or<T: Action>(&self, action: T) -> Result<(), Failure<T>>;
}

impl OptionsValidatorExt for OptionsValidator {
    fn check_or<T: Action>(&self, action: T) -> Result<(), Failure<T>> {
        if !self.check() {
            return Err(Failure::from_action(action));
        }
        Ok(())
    }
}
