use crate::testing_prelude::*;
use claxon::metadata::StreamInfo;

fn valid_stream_info() -> StreamInfo {
    StreamInfo {
        sample_rate: 44100,
        channels: 2,
        bits_per_sample: 16,
        min_block_size: 4096,
        max_block_size: 4096,
        min_frame_size: None,
        max_frame_size: None,
        samples: Some(44100 * 300),
        md5sum: [0; 16],
    }
}

fn path() -> &'static Path {
    Path::new("/tmp/test.flac")
}

#[test]
fn check_sample_rate_valid_44100() {
    let info = valid_stream_info();
    assert_eq!(check_sample_rate(path(), &info), None);
}

#[test]
fn check_sample_rate_valid_48000() {
    let mut info = valid_stream_info();
    info.sample_rate = 96000;
    assert_eq!(check_sample_rate(path(), &info), None);
}

#[test]
fn check_sample_rate_invalid() {
    let mut info = valid_stream_info();
    info.sample_rate = 22050;
    assert_eq!(
        check_sample_rate(path(), &info),
        Some(SourceIssue::SampleRate {
            path: path().to_path_buf(),
            rate: 22050,
        })
    );
}

#[test]
fn check_bit_rate_above_minimum() {
    let info = valid_stream_info();
    assert_eq!(check_bit_rate(path(), &info), None);
}

#[test]
fn check_bit_rate_below_minimum() {
    let mut info = valid_stream_info();
    info.bits_per_sample = 4;
    info.channels = 1;
    assert_eq!(
        check_bit_rate(path(), &info),
        Some(SourceIssue::BitRate {
            path: path().to_path_buf(),
            rate: 176_400,
        })
    );
}

#[test]
fn check_bit_rate_no_samples() {
    let mut info = valid_stream_info();
    info.samples = None;
    assert_eq!(check_bit_rate(path(), &info), None);
}

#[test]
fn check_duration_within_limit() {
    let info = valid_stream_info();
    assert_eq!(check_duration(path(), &info), None);
}

#[test]
fn check_duration_exceeds_limit() {
    let mut info = valid_stream_info();
    info.samples = Some(44100 * 50000);
    assert_eq!(
        check_duration(path(), &info),
        Some(SourceIssue::Duration {
            path: path().to_path_buf(),
            seconds: 50000,
        })
    );
}

#[test]
fn check_duration_no_samples() {
    let mut info = valid_stream_info();
    info.samples = None;
    assert_eq!(check_duration(path(), &info), None);
}

#[test]
fn check_channels_stereo() {
    let info = valid_stream_info();
    assert_eq!(check_channels(path(), &info), None);
}

#[test]
fn check_channels_mono() {
    let mut info = valid_stream_info();
    info.channels = 1;
    assert_eq!(check_channels(path(), &info), None);
}

#[test]
fn check_channels_surround() {
    let mut info = valid_stream_info();
    info.channels = 6;
    assert_eq!(
        check_channels(path(), &info),
        Some(SourceIssue::Channels {
            path: path().to_path_buf(),
            count: 6,
        })
    );
}
