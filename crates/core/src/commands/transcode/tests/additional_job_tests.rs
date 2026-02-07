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
            quality: FileOptions::DEFAULT_JPG_QUALITY,
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
            quality: FileOptions::DEFAULT_JPG_QUALITY,
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
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };

    // Act
    let result = job.execute().await;

    // Assert
    assert!(result.is_err());
    let err = result.expect_err("should be error");
    assert_eq!(err.action(), &TranscodeAction::ResizeImage);
}

/// RGBA PNG is converted to RGB8 when output is JPG.
#[tokio::test]
async fn rgba_png_to_jpg_is_converted() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    let input_path = source_dir.join("cover.png");
    let img = image::RgbaImage::from_fn(600, 450, |x, _y| {
        image::Rgba([u8::try_from(x % 256).expect("should fit"), 0, 128, 128])
    });
    img.save(&input_path).expect("should save RGBA PNG");
    let output_path = output_dir.join("cover.jpg");
    let job = AdditionalJob {
        id: "cover.png".to_owned(),
        resize: Resize {
            input: input_path,
            output: output_path.clone(),
            max_pixel_size: 320,
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };

    // Act
    job.execute().await.expect("should succeed");

    // Assert
    let snapshot = DirectorySnapshot::new()
        .with_directory(&output_dir)
        .create()
        .expect("should read output directory");
    assert_yaml_snapshot!(snapshot);
}

/// 16-bit PNG is converted to RGB8 when output is JPG.
#[tokio::test]
async fn rgb16_png_to_jpg_is_converted() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    let input_path = source_dir.join("cover.png");
    let img = image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_fn(600, 450, |x, _y| {
        image::Rgb([u16::try_from(x % 256).expect("should fit") * 257, 0, 32768])
    });
    img.save(&input_path).expect("should save 16-bit PNG");
    let output_path = output_dir.join("cover.jpg");
    let job = AdditionalJob {
        id: "cover.png".to_owned(),
        resize: Resize {
            input: input_path,
            output: output_path.clone(),
            max_pixel_size: 320,
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };

    // Act
    job.execute().await.expect("should succeed");

    // Assert
    let snapshot = DirectorySnapshot::new()
        .with_directory(&output_dir)
        .create()
        .expect("should read output directory");
    assert_yaml_snapshot!(snapshot);
}

/// Image within `max_pixel_size` is re-encoded without resizing.
#[tokio::test]
async fn small_image_is_not_resized() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    ImageGenerator::new()
        .with_size(200, 150)
        .with_filename("cover.jpg")
        .generate(&source_dir)
        .expect("should generate image");
    let output_path = output_dir.join("cover.jpg");
    let job = AdditionalJob {
        id: "cover.jpg".to_owned(),
        resize: Resize {
            input: source_dir.join("cover.jpg"),
            output: output_path.clone(),
            max_pixel_size: 320,
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };

    // Act
    job.execute().await.expect("should succeed");

    // Assert
    let snapshot = DirectorySnapshot::new()
        .with_directory(&output_dir)
        .create()
        .expect("should read output directory");
    assert_yaml_snapshot!(snapshot);
}

/// Real-world high-res image is resized to under 1 MB.
#[tokio::test]
#[ignore = "downloads image from wikimedia"]
#[expect(
    clippy::integer_division,
    reason = "truncated KB is fine for diagnostic output"
)]
async fn real_world_image_is_under_1mb() {
    // Arrange
    let _ = init_logger();
    let test_dir = TestDirectory::new();
    let source_dir = test_dir.join("source");
    let output_dir = test_dir.join("output");
    fs::create_dir_all(&source_dir).expect("should create source dir");
    fs::create_dir_all(&output_dir).expect("should create output dir");
    let cache_dir = PathBuf::from("/tmp/caesura/image_cache");
    let cached_path = cache_dir.join("abbey_road.jpg");
    if cached_path.exists() {
        eprintln!("Cached: {}", cached_path.display());
    } else {
        fs::create_dir_all(&cache_dir).expect("should create cache dir");
        let url = "https://upload.wikimedia.org/wikipedia/commons/a/a4/The_Beatles_Abbey_Road_album_cover.jpg";
        let client = reqwest::Client::builder()
            .user_agent("caesura-test/0.1 (https://github.com/RogueOneEcho/caesura)")
            .build()
            .expect("should build client");
        let bytes = client
            .get(url)
            .send()
            .await
            .expect("should fetch image")
            .bytes()
            .await
            .expect("should read bytes");
        fs::write(&cached_path, &bytes).expect("should write cached image");
        eprintln!("Downloaded: {}", cached_path.display());
    }
    let input_path = source_dir.join("cover.jpg");
    fs::copy(&cached_path, &input_path).expect("should copy cached image");
    let input_size = fs::metadata(&input_path)
        .expect("should read metadata")
        .len();
    let input_size = usize::try_from(input_size).expect("input size should fit in usize");
    let input_img = image::open(&input_path).expect("should open input");
    let job = AdditionalJob {
        id: "cover.jpg".to_owned(),
        resize: Resize {
            input: input_path,
            output: output_dir.join("cover.jpg"),
            max_pixel_size: FileOptions::DEFAULT_MAX_PIXEL_SIZE,
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };

    // Act
    job.execute().await.expect("should execute");

    // Preview
    let output_path = output_dir.join("cover.jpg");
    let output_size = fs::metadata(&output_path)
        .expect("output should exist")
        .len();
    let output_img = image::open(&output_path).expect("should open output");
    let output_size_usize = usize::try_from(output_size).expect("output size should fit in usize");
    eprintln!(
        "Original: {}x{}, {} KB",
        input_img.width(),
        input_img.height(),
        input_size / 1024,
    );
    eprintln!(
        "Resized: {}x{}, {} KB, {}% of original",
        output_img.width(),
        output_img.height(),
        output_size / 1024,
        output_size_usize * 100 / input_size,
    );
    eprintln!("Resized: {}", output_path.display());

    // Assert
    let max = FileOptions::DEFAULT_MAX_PIXEL_SIZE;
    assert!(
        output_img.width() <= max,
        "width {} exceeds {max}",
        output_img.width()
    );
    assert!(
        output_img.height() <= max,
        "height {} exceeds {max}",
        output_img.height()
    );
    assert!(
        output_size < 1_000_000,
        "output size {output_size} bytes exceeds 1 MB"
    );
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
            quality: FileOptions::DEFAULT_JPG_QUALITY,
        },
    };
    ResizeContext {
        _test_dir: test_dir,
        output_dir,
        job,
    }
}
