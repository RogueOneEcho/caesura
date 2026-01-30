//! Common imports for internal use.

// Internal modules (wildcard re-exports)
pub(crate) use crate::commands::*;
pub(crate) use crate::dependencies::*;
pub(crate) use crate::options::*;
pub(crate) use crate::utils::*;

// External crates
pub(crate) use colored::Colorize;
pub(crate) use di::{Ref, RefMut, inject, injectable};
pub(crate) use log::{debug, error, info, trace, warn};
pub(crate) use rogue_logging::{Colors, Error};

// Std library
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as FmtWrite};
pub(crate) use std::path::{Path, PathBuf};
