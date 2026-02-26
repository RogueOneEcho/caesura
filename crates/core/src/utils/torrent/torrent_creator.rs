//! Create and duplicate `.torrent` files.

use crate::prelude::*;
use lava_torrent::bencode::BencodeElem;
use lava_torrent::torrent::v1::TorrentBuilder;
use rogue_logging::Failure;
use std::collections::HashMap;
use std::fs;
use tokio::fs::copy;
use tokio::task::spawn_blocking;

use super::{TorrentCreateAction, TorrentExt, TorrentReader};

/// Maximum number of threads used by `lava_torrent`.
///
/// `lava_torrent` creates a rayon thread pool for piece hashing. If no default is provided it
/// defaults to the number of CPUs. Normally this would be fine but our test infrastructure calls
/// `TorrentCreator::create()` when generating samples and for various tests. When running on a
/// a machine with 128 cores with tests running in parallel the OS process limit quickly becomes
/// exhausted resulting in:
///
/// ```text
/// TorrentBuilderFailure("failed to create rayon thread pool: Resource temporarily unavailable (os error 11)")
/// ```
const MAX_THREADS: usize = 4;

/// Create and duplicate `.torrent` files using `lava_torrent`.
pub struct TorrentCreator;

impl TorrentCreator {
    /// Create a `.torrent` file for the given content directory.
    ///
    /// - Uses `spawn_blocking` because `lava_torrent` performs synchronous file I/O
    ///   and CPU-intensive piece hashing
    /// - Sets `creation date` to 0 for deterministic output (the field has no practical use)
    /// - Sets `created by` to identify caesura as the creator
    pub async fn create(
        content_dir: &Path,
        output_path: &Path,
        announce_url: String,
        source: String,
    ) -> Result<(), Failure<TorrentCreateAction>> {
        Self::create_with_name(content_dir, output_path, announce_url, source, None).await
    }

    /// Create a `.torrent` file for the given content directory, optionally overriding `info.name`.
    pub async fn create_with_name(
        content_dir: &Path,
        output_path: &Path,
        announce_url: String,
        source: String,
        torrent_name: Option<String>,
    ) -> Result<(), Failure<TorrentCreateAction>> {
        let content_dir = content_dir.to_path_buf();
        let output_path = output_path.to_path_buf();
        spawn_blocking(move || {
            let content_size = dir_size(&content_dir)?;
            let pl = piece_length(content_size);
            let created_by = format!("{APP_NAME} {}", app_version_or_describe());
            let builder = TorrentBuilder::new(&content_dir, pl)
                .set_num_threads(num_threads())
                .set_announce(Some(announce_url))
                .set_privacy(true)
                .add_extra_field("created by".to_owned(), BencodeElem::String(created_by))
                .add_extra_field("creation date".to_owned(), BencodeElem::Integer(0))
                .add_extra_info_field(
                    "source".to_owned(),
                    BencodeElem::String(source.to_uppercase()),
                );
            let builder = if let Some(name) = torrent_name {
                builder.set_name(name)
            } else {
                builder
            };
            let torrent = builder.build().map_err(Failure::wrap_with_path(
                TorrentCreateAction::BuildTorrent,
                &content_dir,
            ))?;
            torrent
                .write_into_file(&output_path)
                .map_err(Failure::wrap_with_path(
                    TorrentCreateAction::WriteTorrent,
                    &output_path,
                ))
        })
        .await
        .expect("torrent create task should not panic")
    }

    /// Duplicate a `.torrent` file for a different tracker.
    ///
    /// - Copies if the source and announce already match
    /// - Otherwise rewrites metadata with new source and announce URL
    /// - Uses `spawn_blocking` because `lava_torrent` performs synchronous file I/O
    pub async fn duplicate(
        from: &Path,
        to: &Path,
        announce_url: String,
        source: String,
    ) -> Result<(), Failure<TorrentCreateAction>> {
        let mut torrent = TorrentReader::execute(from)
            .await
            .map_err(Failure::wrap(TorrentCreateAction::ReadTorrent))?;
        let torrent_announce = torrent
            .announce_list
            .as_ref()
            .and_then(|list| list.first())
            .and_then(|tier| tier.first().cloned())
            .or_else(|| torrent.announce.clone());
        if torrent.is_source_equal(&source) && torrent_announce.as_deref() == Some(&announce_url) {
            trace!(
                "{} {:?} to {:?}",
                "Copying".bold(),
                from.file_name(),
                to.file_name()
            );
            copy(&from, &to).await.map_err(Failure::wrap_with_path(
                TorrentCreateAction::CopyTorrent,
                to,
            ))?;
            return Ok(());
        }
        trace!(
            "{} {:?} to {:?}",
            "Rewriting".bold(),
            from.file_name(),
            to.file_name()
        );
        torrent.announce = Some(announce_url);
        torrent.announce_list = None;
        let info_fields = torrent.extra_info_fields.get_or_insert_with(HashMap::new);
        info_fields.insert(
            "source".to_owned(),
            BencodeElem::String(source.to_uppercase()),
        );
        let to = to.to_path_buf();
        spawn_blocking(move || {
            torrent
                .write_into_file(&to)
                .map_err(Failure::wrap_with_path(
                    TorrentCreateAction::WriteTorrent,
                    &to,
                ))
        })
        .await
        .expect("torrent write task should not panic")
    }
}

/// Calculate total size of all files in a directory recursively.
fn dir_size(path: &Path) -> Result<u64, Failure<TorrentCreateAction>> {
    let mut total: u64 = 0;
    let entries = fs::read_dir(path).map_err(Failure::wrap_with_path(
        TorrentCreateAction::ReadDirectory,
        path,
    ))?;
    for entry in entries {
        let entry = entry.map_err(Failure::wrap_with_path(
            TorrentCreateAction::ReadDirectory,
            path,
        ))?;
        let entry_path = entry.path();
        let ft = entry.file_type().map_err(Failure::wrap_with_path(
            TorrentCreateAction::ReadMetadata,
            &entry_path,
        ))?;
        if ft.is_file() {
            let meta = entry.metadata().map_err(Failure::wrap_with_path(
                TorrentCreateAction::ReadMetadata,
                &entry_path,
            ))?;
            total += meta.len();
        } else if ft.is_dir() {
            total += dir_size(&entry_path)?;
        }
    }
    Ok(total)
}

/// Select piece length.
///
/// Follows the same conventions as
/// [IMDL](https://github.com/casey/intermodal/blob/d565e8c6d78e35dbaaf3eb7432c233bcd09e9bb3/src/piece_length_picker.rs#L17-L20).
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::integer_division,
    clippy::as_conversions,
    reason = "floating point used only for log2 approximation on content size; result is clamped to a safe range"
)]
fn piece_length(content_size: u64) -> i64 {
    let log2 = (content_size.max(1) as f64).log2().ceil() as u32;
    let half_exp = log2 / 2 + 4;
    let length = 1_u64 << half_exp;
    length.clamp(16 * 1024, 16 * 1024 * 1024) as i64
}

fn num_threads() -> usize {
    num_cpus::get().min(MAX_THREADS)
}

#[expect(
    non_upper_case_globals,
    reason = "KiB/MiB/GiB match conventional casing for readability"
)]
#[cfg(test)]
mod tests {
    use super::*;

    const KiB: i64 = 1024;
    const MiB: i64 = KiB * 1024;
    const GiB: i64 = MiB * 1024;

    #[test]
    #[rustfmt::skip]
    #[expect(clippy::identity_op)]
    fn _piece_length() {
        assert_eq!(length_and_count(         1), ( 16 * KiB,      1));
        assert_eq!(length_and_count(   1 * MiB), ( 16 * KiB,     64));
        assert_eq!(length_and_count(   5 * MiB), ( 32 * KiB,    160));
        assert_eq!(length_and_count(  10 * MiB), ( 64 * KiB,    160));
        assert_eq!(length_and_count(  50 * MiB), (128 * KiB,    400));
        assert_eq!(length_and_count( 100 * MiB), (128 * KiB,    800));
        assert_eq!(length_and_count( 150 * MiB), (256 * KiB,    600));
        assert_eq!(length_and_count( 250 * MiB), (256 * KiB,  1_000));
        assert_eq!(length_and_count( 500 * MiB), (256 * KiB,  2_000));
        assert_eq!(length_and_count( 750 * MiB), (512 * KiB,  1_500));
        assert_eq!(length_and_count(   1 * GiB), (512 * KiB,  2_048));
        assert_eq!(length_and_count(   2 * GiB), (512 * KiB,  4_096));
        assert_eq!(length_and_count(   5 * GiB), (  1 * MiB,  5_120));
        assert_eq!(length_and_count(  10 * GiB), (  2 * MiB,  5_120));
        assert_eq!(length_and_count(  15 * GiB), (  2 * MiB,  7_680));
        assert_eq!(length_and_count(  20 * GiB), (  2 * MiB, 10_240));
        assert_eq!(length_and_count(  50 * GiB), (  4 * MiB, 12_800));
        assert_eq!(length_and_count( 100 * GiB), (  4 * MiB, 25_600));
        assert_eq!(length_and_count( 500 * GiB), (  8 * MiB, 64_000));
        assert_eq!(length_and_count(1024 * GiB), ( 16 * MiB, 65_536));
        assert_eq!(piece_length(      u64::MAX),   16 * MiB);
    }

    #[expect(
        clippy::as_conversions,
        clippy::cast_sign_loss,
        clippy::integer_division
    )]
    fn length_and_count(size: i64) -> (i64, i64) {
        let length = piece_length(size as u64);
        let count = (size + length - 1) / length;
        (length, count)
    }
}
