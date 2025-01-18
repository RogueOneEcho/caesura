use crate::utils::testing::*;
use rogue_logging::Error;

pub(crate) fn get_16bit_sample() -> Sample {
    Sample {
        name: "16 bit".to_owned(),
        url: "https://archive.org/download/tennyson-discography_/Tennyson%20-%20With%20You%20-%20Lay-by.zip".to_owned(),
        sha256: "c599d74d09ce6c13b6f04a5bb050e62f1354215360757b97b7608978d2bba2cb".to_owned(),
        dir_name: "Tennyson - With You (2014) [Digital] {16-44.1 Bandcamp} (FLAC)".to_owned(),
    }
}

#[tokio::test]
#[ignore]
async fn fetch_16bit_sample() -> Result<(), Error> {
    // Arrange
    let _ = init_logger();
    let sample = get_16bit_sample();
    assert!(!sample.exists());

    // Act
    sample.fetch().await?;

    // Assert
    assert!(sample.exists());

    Ok(())
}
