use std::env::temp_dir;
use std::path::PathBuf;
use std::time::SystemTime;

pub const TORRENTS_SAMPLES_DIR: &str = "samples/torrents";
pub const CONTENT_SAMPLES_DIR: &str = "samples/content";

pub struct TempDirectory;

impl TempDirectory {
    #[must_use]
    pub fn get(sub_dir_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration should be valid")
            .as_secs()
            .to_string();
        temp_dir().join(sub_dir_name).join(timestamp)
    }

    #[must_use]
    pub fn create(sub_dir_name: &str) -> PathBuf {
        let dir = Self::get(sub_dir_name);
        std::fs::create_dir_all(&dir).expect("Should be able to create temp dir");
        dir
    }
}
