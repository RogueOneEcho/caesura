use crate::utils::testing::*;
use insta::assert_yaml_snapshot;

async fn test_sample_format_determinism(format: SampleFormat, name: &str) {
    let temp_dir = TempDirectory::create(name);

    // Generate samples in temp directory (bypassing the shared cache)
    SampleDataBuilder::new(format)
        .with_directory(&temp_dir)
        .build()
        .await
        .expect("should build samples");

    // Create snapshot excluding torrent (has timestamps) and marker files
    let snapshot = DirectorySnapshot::new()
        .with_directory(&temp_dir)
        .without_extensions(&["torrent"])
        .create()
        .expect("should create snapshot");

    assert_yaml_snapshot!(name, snapshot);
}

#[tokio::test]
#[ignore = "Determinism test - run manually to verify sample generation"]
async fn sample_flac16_441_determinism() {
    test_sample_format_determinism(SampleFormat::FLAC16_441, "sample_flac16_441").await;
}

#[tokio::test]
#[ignore = "Determinism test - run manually to verify sample generation"]
async fn sample_flac16_48_determinism() {
    test_sample_format_determinism(SampleFormat::FLAC16_48, "sample_flac16_48").await;
}

#[tokio::test]
#[ignore = "Determinism test - run manually to verify sample generation"]
async fn sample_flac24_441_determinism() {
    test_sample_format_determinism(SampleFormat::FLAC24_441, "sample_flac24_441").await;
}

#[tokio::test]
#[ignore = "Determinism test - run manually to verify sample generation"]
async fn sample_flac24_48_determinism() {
    test_sample_format_determinism(SampleFormat::FLAC24_48, "sample_flac24_48").await;
}

#[tokio::test]
#[ignore = "Determinism test - run manually to verify sample generation"]
async fn sample_flac24_96_determinism() {
    test_sample_format_determinism(SampleFormat::FLAC24_96, "sample_flac24_96").await;
}
