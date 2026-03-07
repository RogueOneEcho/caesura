use crate::prelude::*;
use tokio::fs::{copy, hard_link};

/// Result of injecting a torrent file into a client auto-add directory.
pub(crate) struct TorrentInjectionResult {
    pub target_path: PathBuf,
    pub verb: &'static str,
}

/// Inject a torrent into an auto-add directory, using hard links if configured.
pub(crate) async fn inject_torrent<T: Debug + Display>(
    source_path: &Path,
    target_dir: &Path,
    hard_link_enabled: bool,
    hard_link_action: T,
    copy_action: T,
) -> Result<TorrentInjectionResult, Failure<T>> {
    let source_file_name = source_path
        .file_name()
        .expect("torrent path should have a file name");
    let target_path = target_dir.join(source_file_name);
    let verb = if hard_link_enabled {
        hard_link(source_path, &target_path)
            .await
            .map_err(Failure::wrap_with_path(hard_link_action, &target_path))?;
        "Hard Linked"
    } else {
        copy(source_path, &target_path)
            .await
            .map_err(Failure::wrap_with_path(copy_action, &target_path))?;
        "Copied"
    };
    Ok(TorrentInjectionResult { target_path, verb })
}

/// Inject a torrent and downgrade any injection errors to warnings.
pub(crate) async fn inject_torrent_or_warn<T: Debug + Display>(
    source_path: &Path,
    target_dir: &Path,
    hard_link_enabled: bool,
    hard_link_action: T,
    copy_action: T,
    warnings: &mut Vec<rogue_logging::Error>,
) {
    match inject_torrent(
        source_path,
        target_dir,
        hard_link_enabled,
        hard_link_action,
        copy_action,
    )
    .await
    {
        Ok(result) => {
            trace!(
                "{} {} to {}",
                result.verb.bold(),
                source_path.display(),
                result.target_path.display()
            );
        }
        Err(error) => {
            warn!("{}", error.render());
            warnings.push(error.to_error());
        }
    }
}
