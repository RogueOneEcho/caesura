use std::fs;

use crate::testing_prelude::*;

/// Large JPG is resized to fit within `max_pixel_size` while preserving aspect ratio.
#[tokio::test]
async fn large_jpg_is_resized() {
    // Arrange
    let ctx = setup_resize("cover.jpg", "cover.jpg");

    // Act
    let snapshot = ctx.execute().await;

    // Assert
    assert_yaml_snapshot!(snapshot);
}

/// Large PNG is converted to JPG and resized.
#[tokio::test]
async fn large_png_is_converted_to_jpg() {
    // Arrange
    let ctx = setup_resize("cover.png", "cover.jpg");

    // Act
    let snapshot = ctx.execute().await;

    // Assert
    assert_yaml_snapshot!(snapshot);
}

/// Returns error when input file does not exist.
#[tokio::test]
async fn error_when_input_file_missing() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    let job = AdditionalJob {
        id: "missing.jpg".to_owned(),
        resize: Resize {
            input: source_dir.join("missing.jpg"),
            output: output_dir.join("output.jpg"),
            max_pixel_size: 320,
            quality: 80,
        },
    };

    // Act
    let result = job.execute().await;

    // Assert
    assert!(result.is_err());
    let err = result.expect_err("should be error");
    assert_eq!(err.action(), &TranscodeAction::ResizeImage);
}

/// Returns error when output directory does not exist.
#[tokio::test]
async fn error_when_output_directory_missing() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    ImageGenerator::new()
        .with_size(600, 450)
        .with_filename("cover.jpg")
        .generate(&source_dir)
        .expect("should generate image");
    let job = AdditionalJob {
        id: "cover.jpg".to_owned(),
        resize: Resize {
            input: source_dir.join("cover.jpg"),
            output: test_dir.join("nonexistent").join("output.jpg"),
            max_pixel_size: 320,
            quality: 80,
        },
    };

    // Act
    let result = job.execute().await;

    // Assert
    assert!(result.is_err());
    let err = result.expect_err("should be error");
    assert_eq!(err.action(), &TranscodeAction::ResizeImage);
}

/// Returns error when input file is not a valid image.
#[tokio::test]
async fn error_when_input_file_invalid() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    let invalid_file = source_dir.join("invalid.jpg");
    fs::write(&invalid_file, b"not an image").expect("should write invalid file");
    let job = AdditionalJob {
        id: "invalid.jpg".to_owned(),
        resize: Resize {
            input: invalid_file,
            output: output_dir.join("output.jpg"),
            max_pixel_size: 320,
            quality: 80,
        },
    };

    // Act
    let result = job.execute().await;

    // Assert
    assert!(result.is_err());
    let err = result.expect_err("should be error");
    assert_eq!(err.action(), &TranscodeAction::ResizeImage);
}

struct ResizeContext {
    _test_dir: TestDirectory,
    output_dir: PathBuf,
    job: AdditionalJob,
}

impl ResizeContext {
    async fn execute(self) -> Vec<FileSnapshot> {
        self.job.execute().await.expect("should execute");
        DirectorySnapshot::new()
            .with_directory(&self.output_dir)
            .create()
            .expect("should read output directory")
    }
}

fn setup_resize(source_filename: &str, output_filename: &str) -> ResizeContext {
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    ImageGenerator::new()
        .with_size(600, 450)
        .with_filename(source_filename)
        .generate(&source_dir)
        .expect("should generate image");
    let job = AdditionalJob {
        id: source_filename.to_owned(),
        resize: Resize {
            input: source_dir.join(source_filename),
            output: output_dir.join(output_filename),
            max_pixel_size: 320,
            quality: 80,
        },
    };
    ResizeContext {
        _test_dir: test_dir,
        output_dir,
        job,
    }
}
