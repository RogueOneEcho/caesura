use crate::built_info::PKG_NAME;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::SystemTime;

pub const TORRENTS_SAMPLES_DIR: &str = "samples/torrents";

pub struct TempDirectory;

impl TempDirectory {
    #[must_use]
    pub fn get(test_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration should be valid")
            .as_millis()
            .to_string();
        temp_dir().join(PKG_NAME).join(test_name).join(timestamp)
    }

    #[must_use]
    pub fn create(test_name: &str) -> PathBuf {
        let dir = Self::get(test_name);
        create_dir_all(&dir).expect("Should be able to create temp dir");
        dir
    }
}
