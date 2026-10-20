//! Extension trait for walking an error's source chain.

use crate::prelude::*;
use std::iter;

/// Extension trait for walking the [`Error::source`] chain.
pub(crate) trait ErrorExt {
    /// Iterator over this error followed by each nested source.
    fn chain(&self) -> impl Iterator<Item = &(dyn Error + 'static)>;

    /// Get the first error of type `T` in the chain.
    ///
    /// - Includes this error itself
    fn get_source<T: Error + 'static>(&self) -> Option<&T>;
}

impl<E: Error + 'static> ErrorExt for E {
    fn chain(&self) -> impl Iterator<Item = &(dyn Error + 'static)> {
        let head: &(dyn Error + 'static) = self;
        iter::successors(Some(head), |&error| error.source())
    }

    fn get_source<T: Error + 'static>(&self) -> Option<&T> {
        self.chain().find_map(|error| error.downcast_ref::<T>())
    }
}
