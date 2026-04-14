use crate::utils::TempDirectory;
use std::fs::remove_dir_all;

/// Test that `TempDirectory::create` creates the directory.
#[test]
fn temp_directory_create() {
    let temp = TempDirectory::create("create_test");
    assert!(temp.exists(), "directory should exist after create");
}

/// Test that directory is deleted on drop.
#[test]
fn temp_directory_drop() {
    let path = {
        let temp = TempDirectory::create("drop_test");
        assert!(temp.exists());
        temp.to_path_buf()
    };
    assert!(!path.exists(), "directory should be deleted after drop");
}

/// Test that `keep()` prevents deletion on drop.
#[test]
fn temp_directory_keep() {
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
fn temp_directory_to_path_buf() {
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
fn temp_directory_deref() {
    let temp = TempDirectory::create("deref_test");
    let joined = temp.join("subdir");
    assert!(joined.starts_with(&temp));
}

/// Test that each call creates a unique directory.
#[test]
fn temp_directory_create_unique_paths() {
    let temp1 = TempDirectory::create("unique_test");
    let temp2 = TempDirectory::create("unique_test");
    assert_ne!(
        temp1.to_path_buf(),
        temp2.to_path_buf(),
        "paths should be unique"
    );
}
