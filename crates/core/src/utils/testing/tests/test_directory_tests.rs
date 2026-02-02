use crate::utils::TestDirectory;
use std::fs::remove_dir_all;

/// Test that `TestDirectory::new` creates the directory.
#[test]
fn new_creates_directory() {
    let test_dir = TestDirectory::new();
    assert!(test_dir.exists(), "directory should exist after new");
}

/// Test that directory is deleted on drop.
#[test]
fn drop_deletes_directory() {
    let path = {
        let test_dir = TestDirectory::new();
        assert!(test_dir.exists());
        test_dir.to_path_buf()
    };
    assert!(!path.exists(), "directory should be deleted after drop");
}

/// Test that `keep()` prevents deletion on drop.
#[test]
fn keep_prevents_deletion() {
    let path = {
        let test_dir = TestDirectory::new().keep();
        assert!(test_dir.exists());
        test_dir.to_path_buf()
    };
    assert!(path.exists(), "directory should still exist after drop");
    remove_dir_all(&path).expect("cleanup");
}

/// Test that `output()` returns correct subdirectory path.
#[test]
fn output_returns_subdirectory() {
    let test_dir = TestDirectory::new();
    let output = test_dir.output();
    assert!(output.ends_with("output"));
    assert!(output.starts_with(&test_dir));
}

/// Test that `cache()` returns correct subdirectory path.
#[test]
fn cache_returns_subdirectory() {
    let test_dir = TestDirectory::new();
    let cache = test_dir.cache();
    assert!(cache.ends_with("cache"));
    assert!(cache.starts_with(&test_dir));
}

/// Test that `Deref` allows path operations.
#[test]
fn deref_allows_path_operations() {
    let test_dir = TestDirectory::new();
    let joined = test_dir.join("subdir");
    assert!(joined.starts_with(&test_dir));
}
