use crate::utils::TempDirectory;
use std::fs::remove_dir_all;

/// Test that `TempDirectory::create` creates the directory.
#[test]
fn create_creates_directory() {
    let temp = TempDirectory::create("create_test");
    assert!(temp.exists(), "directory should exist after create");
}

/// Test that directory is deleted on drop.
#[test]
fn drop_deletes_directory() {
    let path = {
        let temp = TempDirectory::create("drop_test");
        assert!(temp.exists());
        temp.to_path_buf()
    };
    assert!(!path.exists(), "directory should be deleted after drop");
}

/// Test that `keep()` prevents deletion on drop.
#[test]
fn keep_prevents_deletion() {
    let path = {
        let temp = TempDirectory::create("keep_test").keep();
        assert!(temp.exists());
        temp.to_path_buf()
    };
    assert!(path.exists(), "directory should still exist after drop");
    remove_dir_all(&path).expect("cleanup");
}

/// Test that `to_path_buf()` does not prevent deletion.
#[test]
fn to_path_buf_allows_deletion() {
    let path = {
        let temp = TempDirectory::create("to_path_buf_test");
        let p = temp.to_path_buf();
        assert!(p.exists());
        p
    };
    assert!(!path.exists(), "directory should be deleted after drop");
}

/// Test that `Deref` allows path operations.
#[test]
fn deref_allows_path_operations() {
    let temp = TempDirectory::create("deref_test");
    let joined = temp.join("subdir");
    assert!(joined.starts_with(&temp));
}

/// Test that each call creates a unique directory.
#[test]
fn create_generates_unique_paths() {
    let temp1 = TempDirectory::create("unique_test");
    let temp2 = TempDirectory::create("unique_test");
    assert_ne!(
        temp1.to_path_buf(),
        temp2.to_path_buf(),
        "paths should be unique"
    );
}
