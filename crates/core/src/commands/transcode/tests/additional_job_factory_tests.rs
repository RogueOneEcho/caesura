use std::fs;

use crate::testing_prelude::*;
use crate::utils::TargetFormat::_320;

/// Small image is copied, not resized.
#[tokio::test]
async fn small_image_is_copied_not_resized() {
    // Arrange
    let file_options = FileOptions {
        max_file_size: FileOptions::DEFAULT_MAX_FILE_SIZE,
        ..small_max_file_options()
    };
    let (_test_dir, source_dir, factory, source) = setup_factory(file_options).await;
    ImageGenerator::new()
        .with_filename("cover.jpg")
        .generate(&source_dir)
        .expect("should generate image");

    // Act
    let file = AdditionalFile::new(source_dir.join("cover.jpg"), &source_dir);
    let jobs = factory
        .create(&[file], &source, _320)
        .await
        .expect("should succeed");

    // Assert
    assert!(jobs.is_empty(), "small image should be copied, not resized");
}

/// Image exceeding `max_file_size` triggers a resize job.
#[tokio::test]
async fn large_image_triggers_resize_job() {
    // Arrange
    let (_test_dir, source_dir, factory, source) = setup_factory(small_max_file_options()).await;
    ImageGenerator::new()
        .with_filename("cover.jpg")
        .generate(&source_dir)
        .expect("should generate image");

    // Act
    let job = create_resize_job(&factory, &source, &source_dir, "cover.jpg").await;

    // Assert
    assert_eq!(job.resize.max_pixel_size, 1280);
    assert_eq!(job.resize.quality, 80);
    assert_eq!(
        job.resize
            .output
            .extension()
            .expect("should have extension")
            .to_str(),
        Some("jpg")
    );
}

/// `no_image_compression` skips resize even for images exceeding `max_file_size`.
#[tokio::test]
async fn no_image_compression_skips_resize() {
    // Arrange
    let file_options = FileOptions {
        no_image_compression: true,
        ..small_max_file_options()
    };
    let (_test_dir, source_dir, factory, source) = setup_factory(file_options).await;
    ImageGenerator::new()
        .with_filename("cover.jpg")
        .generate(&source_dir)
        .expect("should generate image");

    // Act
    let file = AdditionalFile::new(source_dir.join("cover.jpg"), &source_dir);
    let jobs = factory
        .create(&[file], &source, _320)
        .await
        .expect("should succeed");

    // Assert
    assert!(jobs.is_empty(), "no_image_compression should skip resize");
}

/// PNG exceeding `max_file_size` is converted to JPG by default.
#[tokio::test]
async fn large_png_converts_to_jpg() {
    // Arrange
    let (_test_dir, source_dir, factory, source) = setup_factory(small_max_file_options()).await;
    ImageGenerator::new()
        .with_filename("cover.png")
        .generate(&source_dir)
        .expect("should generate image");

    // Act
    let job = create_resize_job(&factory, &source, &source_dir, "cover.png").await;

    // Assert
    assert_eq!(
        job.resize
            .output
            .extension()
            .expect("should have extension")
            .to_str(),
        Some("jpg"),
        "PNG should be converted to JPG"
    );
}

/// `no_png_to_jpg` preserves PNG extension.
#[tokio::test]
async fn no_png_to_jpg_preserves_png() {
    // Arrange
    let file_options = FileOptions {
        no_png_to_jpg: true,
        ..small_max_file_options()
    };
    let (_test_dir, source_dir, factory, source) = setup_factory(file_options).await;
    ImageGenerator::new()
        .with_filename("cover.png")
        .generate(&source_dir)
        .expect("should generate image");

    // Act
    let job = create_resize_job(&factory, &source, &source_dir, "cover.png").await;

    // Assert
    assert_eq!(
        job.resize
            .output
            .extension()
            .expect("should have extension")
            .to_str(),
        Some("png"),
        "PNG extension should be preserved with no_png_to_jpg"
    );
}

/// Text files are always copied, never resized, even when exceeding `max_file_size`.
#[tokio::test]
async fn text_file_is_copied_not_resized() {
    // Arrange
    let (_test_dir, source_dir, factory, source) = setup_factory(small_max_file_options()).await;
    fs::write(source_dir.join("info.txt"), "a".repeat(1000)).expect("should write text file");

    // Act
    let file = AdditionalFile::new(source_dir.join("info.txt"), &source_dir);
    let jobs = factory
        .create(&[file], &source, _320)
        .await
        .expect("should succeed");

    // Assert
    assert!(jobs.is_empty(), "text file should be copied, not resized");
}

fn small_max_file_options() -> FileOptions {
    FileOptions {
        max_file_size: 100,
        no_image_compression: false,
        rename_tracks: false,
        no_png_to_jpg: false,
        max_pixel_size: FileOptions::DEFAULT_MAX_PIXEL_SIZE,
        jpg_quality: FileOptions::DEFAULT_JPG_QUALITY,
    }
}

async fn setup_factory(
    file_options: FileOptions,
) -> (TestDirectory, PathBuf, Ref<AdditionalJobFactory>, Source) {
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(file_options)
        .with_options(CopyOptions { hard_link: false })
        .expect_build();
    let provider = host.services.get_required::<SourceProvider>();
    let factory = host.services.get_required::<AdditionalJobFactory>();
    let source = provider
        .get(AlbumConfig::TORRENT_ID)
        .await
        .expect("should not fail")
        .expect("should find source");
    (test_dir, source_dir, factory, source)
}

async fn create_resize_job(
    factory: &AdditionalJobFactory,
    source: &Source,
    source_dir: &PathBuf,
    filename: &str,
) -> AdditionalJob {
    let file = AdditionalFile::new(source_dir.join(filename), source_dir);
    let mut jobs = factory
        .create(&[file], source, _320)
        .await
        .expect("should succeed");
    assert_eq!(jobs.len(), 1, "expected exactly one job");
    match jobs.remove(0) {
        Job::Additional(job) => job,
        _ => unreachable!("expected Additional job variant"),
    }
}
