use crate::prelude::*;
use tokio::fs::{copy, create_dir, hard_link, read_dir};

/// Copy the contents of one directory to another.
///
/// The target directory will be created if it does not exist, but its parent must exist.
///
/// If `use_hard_link` is true, the files will be hard linked instead of copied.
pub async fn copy_dir(
    source_dir: &Path,
    target_dir: &Path,
    use_hard_link: bool,
) -> Result<(), Failure<FsAction>> {
    if !source_dir.exists() {
        return Err(Failure::new(
            FsAction::CopyDirectory,
            IoError::new(ErrorKind::NotFound, "source directory does not exist"),
        )
        .with_path(source_dir));
    }
    if !source_dir.is_dir() {
        return Err(Failure::new(
            FsAction::CopyDirectory,
            IoError::new(ErrorKind::NotADirectory, "source path is not a directory"),
        )
        .with_path(source_dir));
    }
    let target_parent = target_dir.parent().ok_or_else(|| {
        Failure::new(
            FsAction::CopyDirectory,
            IoError::new(ErrorKind::InvalidInput, "target directory has no parent"),
        )
        .with_path(target_dir)
    })?;
    if !target_parent.exists() {
        return Err(Failure::new(
            FsAction::CopyDirectory,
            IoError::new(
                ErrorKind::NotFound,
                "parent of the target directory does not exist",
            ),
        )
        .with_path(target_parent));
    }
    if target_dir.exists() {
        return Err(Failure::new(
            FsAction::CopyDirectory,
            IoError::new(ErrorKind::AlreadyExists, "target directory already exists"),
        )
        .with_path(target_dir));
    }
    create_dir(target_dir)
        .await
        .map_err(Failure::wrap_with_path(
            FsAction::CreateDirectory,
            target_dir,
        ))?;
    let mut dir = read_dir(source_dir)
        .await
        .map_err(Failure::wrap_with_path(FsAction::ReadDirectory, source_dir))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(Failure::wrap_with_path(FsAction::ReadDirectory, source_dir))?
    {
        let source_entry_path = entry.path();
        let target_path = target_dir.join(entry.file_name());
        if source_entry_path.is_dir() {
            Box::pin(copy_dir(&source_entry_path, &target_path, use_hard_link)).await?;
        } else if use_hard_link {
            hard_link(&source_entry_path, &target_path)
                .await
                .map_err(Failure::wrap_with_path(FsAction::HardLink, &target_path))?;
        } else {
            copy(&source_entry_path, &target_path)
                .await
                .map_err(Failure::wrap_with_path(FsAction::CopyFile, &target_path))?;
        }
    }
    Ok(())
}
