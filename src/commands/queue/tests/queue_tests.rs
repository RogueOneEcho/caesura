use super::super::Queue;

use flat_db::Hash;
use rogue_logging::Error;
use std::path::PathBuf;

use crate::commands::*;
use crate::utils::*;

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

    let mut queue = Queue::from_path(TempDirectory::create("queue_get_unprocessed"));
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
