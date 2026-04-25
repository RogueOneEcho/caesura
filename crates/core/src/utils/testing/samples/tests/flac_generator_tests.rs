use crate::testing_prelude::*;

#[tokio::test]
async fn flac_generator_with_cover_image() {
    let temp_dir = TempDirectory::create("flac_generator_cover");

    let path = FlacGenerator::new()
        .with_filename("test.flac")
        .with_cover_image()
        .generate(&temp_dir)
        .await
        .expect("Should generate FLAC with cover");

    let metadata = AudioSnapshot::from_path(&path).expect("Should read metadata");
    assert_yaml_snapshot!(metadata);
}

#[tokio::test]
async fn flac_generator_generate_omits_vorbis_comments() {
    // Arrange
    let dir = TempDirectory::create("flac_generator_generate_omits_vorbis_comments");
    // Act
    let path = FlacGenerator::new()
        .with_filename("test.flac")
        .omit_vorbis_comments()
        .generate(&dir)
        .await
        .expect("generate should succeed");
    // Assert
    let output = TokioCommand::new(METAFLAC)
        .arg("--list")
        .arg("--block-type=VORBIS_COMMENT")
        .arg(&path)
        .output()
        .await
        .expect("metaflac should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "expected no VORBIS_COMMENT block, got: {stdout}"
    );
}
