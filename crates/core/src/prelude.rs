//! Common imports for internal use.

pub(crate) use crate::commands::*;
pub(crate) use crate::dependencies::*;
pub(crate) use crate::hosting::*;
pub(crate) use crate::options::*;
pub(crate) use crate::utils::*;
pub(crate) use caesura_macros::Options;
pub(crate) use caesura_options::*;
pub(crate) use colored::{ColoredString, Colorize};
pub(crate) use di::{Ref, RefMut, inject, injectable};
pub(crate) use flat_db::{Hash, Table};
pub(crate) use gazelle_api::{
    ApiResponseKind, BrowseRequest, BrowseResponse, BrowseTorrent, Category, FileItem, Format,
    GazelleClientFactory, GazelleClientOptions, GazelleClientTrait, GazelleError, GazelleOperation,
    GazelleSerializableError, Group, Media, Quality, ReleaseType, ReleaseTypeId, Torrent,
    TorrentResponse, UploadForm,
};
pub(crate) use log::{debug, error, info, trace, warn};
pub(crate) use qbittorrent_api::QBittorrentClientTrait;
pub(crate) use regex::Regex;
pub(crate) use rogue_logging::{Colors, Failure, Logger, TimeFormat, Verbosity};
pub(crate) use serde::de::{Error as SerdeError, Visitor};
pub(crate) use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub(crate) use serde_yaml::{
    Error as YamlError, Value, from_reader as yaml_from_reader, from_str as yaml_from_str,
    to_string as yaml_to_string, to_value as yaml_to_value,
};
pub(crate) use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
pub(crate) use std::error::Error;
pub(crate) use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as FmtWrite};
pub(crate) use std::fs::{File, create_dir, create_dir_all, read_dir, read_to_string};
pub(crate) use std::io::{Error as IoError, ErrorKind};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::str::FromStr;
pub(crate) use std::sync::{Arc, LazyLock};
pub(crate) use std::time::Duration;
pub(crate) use thiserror::Error as ThisError;
pub(crate) use tokio::fs::File as TokioFile;
pub(crate) use tokio::fs::copy as tokio_copy;
pub(crate) use tokio::fs::create_dir as tokio_create_dir;
pub(crate) use tokio::fs::create_dir_all as tokio_create_dir_all;
pub(crate) use tokio::fs::hard_link as tokio_hard_link;
pub(crate) use tokio::fs::read_dir as tokio_read_dir;
pub(crate) use tokio::fs::rename as tokio_rename;
pub(crate) use tokio::process::Command as TokioCommand;
pub(crate) use tokio::sync::Semaphore;
pub(crate) use tokio::task::{JoinSet, spawn_blocking};
pub(crate) use tokio::time::sleep;

/// DI-injected Gazelle API client.
pub(crate) type GazelleClient = Box<dyn GazelleClientTrait + Send + Sync>;

/// DI-injected qBittorrent API client.
pub(crate) type QbitClient = Box<dyn QBittorrentClientTrait + Send + Sync>;
