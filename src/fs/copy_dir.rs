use crate::errors::AppError;
use std::path::Path;
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
) -> Result<(), AppError> {
    if !source_dir.exists() {
        return AppError::explained(
            "copy directory",
            format!("source directory does not exist: {source_dir:?}"),
        );
    }
    if !source_dir.is_dir() {
        return AppError::explained(
            "copy directory",
            format!("source path is not a directory: {source_dir:?}"),
        );
    }
    let target_parent = target_dir.parent().ok_or_else(|| {
        AppError::else_explained(
            "copy directory",
            "target directory has no parent".to_owned(),
        )
    })?;
    if !target_parent.exists() {
        return AppError::explained(
            "copy directory",
            format!("parent of the target directory does not exist: {target_parent:?}"),
        );
    }
    if target_dir.exists() {
        return AppError::explained(
            "copy directory",
            format!("target directory already exists: {target_dir:?}"),
        );
    }
    create_dir(target_dir)
        .await
        .or_else(|e| AppError::io(e, "create target directory"))?;
    let mut dir = read_dir(source_dir)
        .await
        .or_else(|e| AppError::io(e, "read source directory"))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .or_else(|e| AppError::io(e, "read source directory items"))?
    {
        let source_entry_path = entry.path();
        let target_path = target_dir.join(entry.file_name());
        if source_entry_path.is_dir() {
            Box::pin(copy_dir(&source_entry_path, &target_path, use_hard_link)).await?;
        } else if use_hard_link {
            hard_link(&source_entry_path, &target_path)
                .await
                .or_else(|e| AppError::io(e, "hard link file to target directory"))?;
        } else {
            copy(&source_entry_path, &target_path)
                .await
                .or_else(|e| AppError::io(e, "copy file to target directory"))?;
        }
    }
    Ok(())
}
