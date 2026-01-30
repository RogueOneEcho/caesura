#![allow(clippy::indexing_slicing)] // Indexing is safe in tests after length checks

use super::super::Queue;
use crate::testing_prelude::*;
use flat_db::Hash;

/// Test that `Queue::set` adds an item successfully.
#[tokio::test]
async fn queue_set_adds_item() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_set_adds_item"));
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let item = QueueItem {
        name: "Test Item".to_owned(),
        path: PathBuf::from("/test/path.torrent"),
        hash,
        indexer: "red".to_owned(),
        ..QueueItem::default()
    };

    // Act
    queue.set(item.clone()).await?;

    // Assert
    let retrieved = queue.get(hash)?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.expect("should have item");
    assert_eq!(retrieved.name, "Test Item");
    assert_eq!(retrieved.indexer, "red");
    Ok(())
}

/// Test that `Queue::set_many` adds multiple items.
#[tokio::test]
async fn queue_set_many_adds_items() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_set_many_adds_items"));
    let hash1 = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let hash2 = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;

    let items = BTreeMap::from([
        (
            hash1,
            QueueItem {
                name: "Item One".to_owned(),
                path: PathBuf::from("/test/one.torrent"),
                hash: hash1,
                indexer: "red".to_owned(),
                ..QueueItem::default()
            },
        ),
        (
            hash2,
            QueueItem {
                name: "Item Two".to_owned(),
                path: PathBuf::from("/test/two.torrent"),
                hash: hash2,
                indexer: "red".to_owned(),
                ..QueueItem::default()
            },
        ),
    ]);

    // Act
    let added = queue.set_many(items, false).await?;

    // Assert
    assert_eq!(added, 2);
    assert!(queue.get(hash1)?.is_some());
    assert!(queue.get(hash2)?.is_some());
    Ok(())
}

/// Test that `Queue::set_many` with replace=false does not overwrite existing items.
#[tokio::test]
async fn queue_set_many_no_replace_skips_existing() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_set_many_no_replace"));
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;

    let original = QueueItem {
        name: "Original".to_owned(),
        path: PathBuf::from("/test/original.torrent"),
        hash,
        indexer: "red".to_owned(),
        ..QueueItem::default()
    };
    queue.set(original).await?;

    let updated = BTreeMap::from([(
        hash,
        QueueItem {
            name: "Updated".to_owned(),
            path: PathBuf::from("/test/updated.torrent"),
            hash,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        },
    )]);

    // Act
    let added = queue.set_many(updated, false).await?;

    // Assert
    assert_eq!(added, 0); // Should not add since item exists
    let retrieved = queue.get(hash)?.expect("should have item");
    assert_eq!(retrieved.name, "Original"); // Should keep original
    Ok(())
}

/// Test that `Queue::set_many` with replace=true overwrites existing items.
#[tokio::test]
async fn queue_set_many_with_replace_overwrites() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_set_many_with_replace"));
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;

    let original = QueueItem {
        name: "Original".to_owned(),
        path: PathBuf::from("/test/original.torrent"),
        hash,
        indexer: "red".to_owned(),
        ..QueueItem::default()
    };
    queue.set(original).await?;

    let updated = BTreeMap::from([(
        hash,
        QueueItem {
            name: "Updated".to_owned(),
            path: PathBuf::from("/test/updated.torrent"),
            hash,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        },
    )]);

    // Act
    let added = queue.set_many(updated, true).await?;

    // Assert
    assert_eq!(added, 1); // Should replace
    let retrieved = queue.get(hash)?.expect("should have item");
    assert_eq!(retrieved.name, "Updated"); // Should have new name
    Ok(())
}

/// Test that `Queue::get_all` returns all items.
#[tokio::test]
async fn queue_get_all_returns_all_items() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_get_all"));
    let hash1 = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let hash2 = Hash::<20>::from_string("ff00000000000000000000000000000000000000")?;

    queue
        .set(QueueItem {
            name: "Item 1".to_owned(),
            hash: hash1,
            path: PathBuf::new(),
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "Item 2".to_owned(),
            hash: hash2,
            path: PathBuf::new(),
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let all = queue.get_all().await?;

    // Assert
    assert_eq!(all.len(), 2);
    Ok(())
}

/// Test that `Queue::remove` removes an existing item.
#[tokio::test]
async fn queue_remove_removes_existing_item() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_remove_existing"));
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let item = QueueItem {
        name: "Test Item".to_owned(),
        path: PathBuf::from("/test/path.torrent"),
        hash,
        indexer: "red".to_owned(),
        ..QueueItem::default()
    };
    queue.set(item).await?;

    // Verify item exists
    assert!(queue.get(hash)?.is_some());

    // Act
    let removed = queue.remove(hash).await?;

    // Assert
    assert!(removed.is_some());
    let removed = removed.expect("should have removed item");
    assert_eq!(removed.name, "Test Item");

    // Verify item no longer exists
    assert!(queue.get(hash)?.is_none());
    Ok(())
}

/// Test that `Queue::remove` returns None for non-existent item.
#[tokio::test]
async fn queue_remove_nonexistent_returns_none() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_remove_nonexistent"));
    let hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;

    // Act
    let removed = queue.remove(hash).await?;

    // Assert
    assert!(removed.is_none());
    Ok(())
}

/// Test that `Queue::remove` only removes the specified item.
#[tokio::test]
async fn queue_remove_only_affects_specified_item() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_remove_specific"));
    let hash1 = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let hash2 = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;

    queue
        .set(QueueItem {
            name: "Item One".to_owned(),
            path: PathBuf::from("/test/one.torrent"),
            hash: hash1,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "Item Two".to_owned(),
            path: PathBuf::from("/test/two.torrent"),
            hash: hash2,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let removed = queue.remove(hash1).await?;

    // Assert
    assert!(removed.is_some());
    assert!(queue.get(hash1)?.is_none()); // First item removed
    assert!(queue.get(hash2)?.is_some()); // Second item still exists
    Ok(())
}

/// Test `get_unprocessed` filters by processing status.
#[tokio::test]
#[allow(deprecated)]
async fn queue_get_unprocessed() -> Result<(), Error> {
    // Arrange
    let new = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let verified = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;
    let not_verified = Hash::<20>::from_string("0300000000000000000000000000000000000000")?;
    let transcoded = Hash::<20>::from_string("0400000000000000000000000000000000000000")?;
    let not_transcoded = Hash::<20>::from_string("0500000000000000000000000000000000000000")?;
    let uploaded = Hash::<20>::from_string("0600000000000000000000000000000000000000")?;
    let not_uploaded = Hash::<20>::from_string("0700000000000000000000000000000000000000")?;

    let queue = Queue::from_path(TempDirectory::create("queue_get_unprocessed"));
    queue
        .set(QueueItem {
            name: "NEW".to_owned(),
            path: PathBuf::new(),
            hash: new,
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "VERIFIED".to_owned(),
            path: PathBuf::new(),
            hash: verified,
            verify: Some(VerifyStatus::verified()),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "VERIFY FAILURE".to_owned(),
            path: PathBuf::new(),
            hash: not_verified,
            verify: Some(VerifyStatus::from_issue(SourceIssue::IdError {
                details: "missing id".to_owned(),
            })),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "TRANSCODED".to_owned(),
            path: PathBuf::new(),
            hash: transcoded,
            verify: Some(VerifyStatus::verified()),
            transcode: Some(TranscodeStatus {
                success: true,
                completed: TimeStamp::now(),
                formats: None,
                error: None,
            }),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "TRANSCODE FAILURE".to_owned(),
            path: PathBuf::new(),
            hash: not_transcoded,
            verify: Some(VerifyStatus::verified()),
            transcode: Some(TranscodeStatus {
                success: false,
                completed: TimeStamp::now(),
                formats: None,
                error: None,
            }),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "UPLOADED".to_owned(),
            path: PathBuf::new(),
            hash: uploaded,
            verify: Some(VerifyStatus::verified()),
            transcode: Some(TranscodeStatus {
                success: true,
                completed: TimeStamp::now(),
                formats: None,
                error: None,
            }),
            upload: Some(UploadStatus {
                success: true,
                completed: TimeStamp::now(),
                formats: None,
                errors: None,
            }),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "UPLOAD FAILURE".to_owned(),
            path: PathBuf::new(),
            hash: not_uploaded,
            verify: Some(VerifyStatus::verified()),
            transcode: Some(TranscodeStatus {
                success: true,
                completed: TimeStamp::now(),
                formats: None,
                error: None,
            }),
            upload: Some(UploadStatus {
                success: false,
                completed: TimeStamp::now(),
                formats: None,
                errors: None,
            }),
            ..QueueItem::default()
        })
        .await?;

    // Assert
    let verify = queue
        .get_unprocessed(String::new(), false, false, false)
        .await?;
    assert_eq!(verify, vec![new]);
    let transcode = queue
        .get_unprocessed(String::new(), true, false, false)
        .await?;
    assert_eq!(transcode, vec![new, verified]);
    let transcode_with_failed = queue
        .get_unprocessed(String::new(), true, false, true)
        .await?;
    assert_eq!(transcode_with_failed, vec![new, not_transcoded, verified]);
    let upload = queue
        .get_unprocessed(String::new(), true, true, false)
        .await?;
    assert_eq!(upload, vec![new, transcoded, verified]);
    let upload_with_failed = queue
        .get_unprocessed(String::new(), true, true, true)
        .await?;
    assert_eq!(
        upload_with_failed,
        vec![new, not_transcoded, transcoded, verified]
    );
    Ok(())
}

/// Test `get_unprocessed` filters by indexer.
#[tokio::test]
async fn queue_get_unprocessed_filters_by_indexer() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_filter_indexer"));
    let red_hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let ops_hash = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;

    queue
        .set(QueueItem {
            name: "RED Item".to_owned(),
            path: PathBuf::new(),
            hash: red_hash,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "OPS Item".to_owned(),
            path: PathBuf::new(),
            hash: ops_hash,
            indexer: "ops".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let red_items = queue
        .get_unprocessed("red".to_owned(), false, false, false)
        .await?;
    let ops_items = queue
        .get_unprocessed("ops".to_owned(), false, false, false)
        .await?;

    // Assert
    assert_eq!(red_items.len(), 1);
    assert_eq!(red_items[0], red_hash);
    assert_eq!(ops_items.len(), 1);
    assert_eq!(ops_items[0], ops_hash);
    Ok(())
}

/// Test `get_unprocessed` includes PTH items when indexer is RED.
#[tokio::test]
async fn queue_get_unprocessed_red_includes_pth() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_red_includes_pth"));
    let red_hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let pth_hash = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;

    queue
        .set(QueueItem {
            name: "RED Item".to_owned(),
            path: PathBuf::new(),
            hash: red_hash,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "PTH Item".to_owned(),
            path: PathBuf::new(),
            hash: pth_hash,
            indexer: "pth".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let items = queue
        .get_unprocessed("red".to_owned(), false, false, false)
        .await?;

    // Assert - RED includes both RED and PTH items
    assert_eq!(items.len(), 2);
    Ok(())
}

/// Test `get_unprocessed` sorts items by name.
#[tokio::test]
async fn queue_get_unprocessed_sorts_by_name() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_sorts_by_name"));
    let hash_z = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let hash_a = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;
    let hash_m = Hash::<20>::from_string("0300000000000000000000000000000000000000")?;

    // Insert in non-alphabetical order
    queue
        .set(QueueItem {
            name: "Zebra Album".to_owned(),
            path: PathBuf::new(),
            hash: hash_z,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "Apple Album".to_owned(),
            path: PathBuf::new(),
            hash: hash_a,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "Mango Album".to_owned(),
            path: PathBuf::new(),
            hash: hash_m,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let items = queue
        .get_unprocessed("red".to_owned(), false, false, false)
        .await?;

    // Assert - Should be sorted alphabetically: Apple, Mango, Zebra
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], hash_a); // Apple
    assert_eq!(items[1], hash_m); // Mango
    assert_eq!(items[2], hash_z); // Zebra
    Ok(())
}

/// Test `get_unprocessed` excludes uploaded items.
#[tokio::test]
async fn queue_get_unprocessed_excludes_uploaded() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_excludes_uploaded"));
    let uploaded_hash = Hash::<20>::from_string("0100000000000000000000000000000000000000")?;
    let pending_hash = Hash::<20>::from_string("0200000000000000000000000000000000000000")?;

    queue
        .set(QueueItem {
            name: "Uploaded Item".to_owned(),
            path: PathBuf::new(),
            hash: uploaded_hash,
            indexer: "red".to_owned(),
            upload: Some(UploadStatus {
                success: true,
                completed: TimeStamp::now(),
                formats: None,
                errors: None,
            }),
            ..QueueItem::default()
        })
        .await?;
    queue
        .set(QueueItem {
            name: "Pending Item".to_owned(),
            path: PathBuf::new(),
            hash: pending_hash,
            indexer: "red".to_owned(),
            ..QueueItem::default()
        })
        .await?;

    // Act
    let items = queue
        .get_unprocessed("red".to_owned(), true, true, false)
        .await?;

    // Assert - Should only return pending item
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], pending_hash);
    Ok(())
}

/// Test `get_unprocessed` with empty queue returns empty list.
#[tokio::test]
async fn queue_get_unprocessed_empty_queue() -> Result<(), Error> {
    // Arrange
    let queue = Queue::from_path(TempDirectory::create("queue_empty"));

    // Act
    let items = queue
        .get_unprocessed("red".to_owned(), true, true, false)
        .await?;

    // Assert
    assert!(items.is_empty());
    Ok(())
}
