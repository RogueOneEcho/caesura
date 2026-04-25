//! Test-specific imports extending the main prelude.

pub(crate) use crate::prelude::*;
pub(crate) use gazelle_api::{
    ApiResponseError, Credit, Credits, ErrorSource, GroupResponse, MockGazelleClient,
    TorrentResponse, UploadResponse,
};
pub(crate) use insta::{assert_snapshot, assert_yaml_snapshot};
pub(crate) use serde_yaml::from_str as yaml_from_str;
pub(crate) use std::fs::{copy, metadata, read, remove_dir_all, remove_file, write};
pub(crate) use tokio::sync::OnceCell;

/// Type alias for test error results.
pub(crate) type TestError = Box<dyn Error + Send + Sync>;
