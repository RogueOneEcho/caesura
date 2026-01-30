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
