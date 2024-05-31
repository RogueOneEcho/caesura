use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use di::*;

use crate::fs::{Collector, DirectoryReader};
use crate::hosting::*;
use crate::options::*;
use crate::source::Source;

pub const TORRENTS_SAMPLES_DIR: &str = "samples/torrents";
pub const CONTENT_SAMPLES_DIR: &str = "samples/content";

#[must_use]
pub fn create_shared_options(mut options: SharedOptions) -> SharedOptions {
    let provider = OptionsProvider::new();
    options.merge(&provider.get_shared_options());
    options
}

#[must_use]
pub fn create_spectrogram_options(mut options: SpectrogramOptions) -> SpectrogramOptions {
    let provider = OptionsProvider::new();
    options.merge(&provider.get_spectrogram_options());
    options
}

#[must_use]
pub fn create_transcode_options(mut options: TranscodeOptions) -> TranscodeOptions {
    let provider = OptionsProvider::new();
    options.merge(&provider.get_transcode_options());
    options
}

#[must_use]
pub fn create_host(
    shared_options: SharedOptions,
    spectrogram_options: SpectrogramOptions,
    transcode_options: TranscodeOptions,
) -> Host {
    let mut builder = HostBuilder::new();
    builder
        .services
        .add(singleton_as_self().from(move |_| Ref::new(shared_options.clone())))
        .add(singleton_as_self().from(move |_| Ref::new(spectrogram_options.clone())))
        .add(singleton_as_self().from(move |_| Ref::new(transcode_options.clone())));
    builder.build().expect("Builder should be valid")
}

#[must_use]
pub fn get_temp_dir(sub_dir_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Duration should be valid")
        .as_secs()
        .to_string();
    temp_dir().join(sub_dir_name).join(timestamp)
}

#[must_use]
pub fn create_temp_dir(sub_dir_name: &str) -> PathBuf {
    let dir = get_temp_dir(sub_dir_name);
    std::fs::create_dir_all(&dir).expect("Should be able to create temp dir");
    dir
}

pub fn read_dir_recursive(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    DirectoryReader::new().read(directory)
}

pub fn read_flacs_recursive(directory: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    DirectoryReader::new()
        .with_extension("flac")
        .read(directory)
}

pub fn count_flacs_recursive(directory: &Path) -> Result<usize, std::io::Error> {
    Ok(read_flacs_recursive(directory)?.len())
}

pub fn get_sample_torrent_file() -> Result<PathBuf, std::io::Error> {
    let path = std::fs::read_dir(TORRENTS_SAMPLES_DIR)?
        .map(|entry| entry.expect("directory entry should have a path").path())
        .find(|path| path.is_file() && path.extension().unwrap_or_default() == "torrent")
        .expect("Should be at least one sample torrent file found");
    Ok(path)
}

pub fn get_first_sample_content_dir() -> Result<PathBuf, std::io::Error> {
    let path = std::fs::read_dir(CONTENT_SAMPLES_DIR)?
        .map(|entry| entry.expect("directory entry should have a path").path())
        .find(|path| path.is_dir())
        .expect("Should be at least one directory");
    Ok(path)
}

#[must_use]
pub fn source_file_count(source: &Source) -> usize {
    Collector::get_flacs(&source.directory).len()
}
