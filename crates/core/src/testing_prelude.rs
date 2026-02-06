//! Test-specific imports extending the main prelude.

use std::error::Error;

pub(crate) use crate::hosting::*;
pub(crate) use crate::prelude::*;
pub(crate) use crate::utils::SAMPLE_SOURCES_DIR;
pub(crate) use insta::assert_yaml_snapshot;

/// Type alias for test error results.
pub(crate) type TestError = Box<dyn Error + Send + Sync>;
