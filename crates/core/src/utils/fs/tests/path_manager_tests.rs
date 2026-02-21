//! Tests for [`PathManager`] default path functions and naming modes.

use crate::hosting::HostBuilder;
use crate::prelude::*;
use gazelle_api::{Group, Torrent};

/// Verify default config path ends with the expected platform suffix.
#[test]
fn default_config_path() {
    let path = PathManager::default_config_path();
    let expected = if is_docker() {
        "/config.yml"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".config/caesura/config.yml"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Application Support/caesura/config.yml"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Roaming/caesura/config.yml"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default cache directory ends with the expected platform suffix.
#[test]
fn default_cache_dir() {
    let path = PathManager::default_cache_dir();
    let expected = if is_docker() {
        "/cache"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".cache/caesura"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Caches/caesura"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Local/caesura"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

/// Verify default output directory ends with the expected platform suffix.
#[test]
fn default_output_dir() {
    let path = PathManager::default_output_dir();
    let expected = if is_docker() {
        "/output"
    } else {
        #[cfg(target_os = "linux")]
        {
            ".local/share/caesura/output"
        }
        #[cfg(target_os = "macos")]
        {
            "Library/Application Support/caesura/output"
        }
        #[cfg(target_os = "windows")]
        {
            "AppData/Roaming/caesura/output"
        }
    };
    assert!(
        path.ends_with(expected),
        "expected path to end with '{expected}', got: {path:?}"
    );
}

fn test_source() -> Source {
    Source {
        torrent: Torrent::default(),
        group: Group::default(),
        existing: BTreeSet::new(),
        format: SourceFormat::Flac,
        directory: PathBuf::from("/test"),
        metadata: Metadata::mock(),
    }
}

fn build_path_manager(shared: SharedOptions, name: NameOptions) -> Ref<PathManager> {
    HostBuilder::new()
        .with_options(shared)
        .with_options(name)
        .expect_build()
        .services
        .get_required::<PathManager>()
}

/// Verify default naming matches `SourceName::get()`.
#[test]
fn resolve_name_default() {
    // Arrange
    let source = test_source();
    let paths = build_path_manager(SharedOptions::mock(), NameOptions::default());

    // Act
    let dir = paths.get_transcode_target_dir(&source, TargetFormat::Flac);

    // Assert
    let expected_prefix = SourceName::get(&source.metadata);
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, format!("{expected_prefix} [CD FLAC]"));
}

/// Verify static `--name` override is used as folder prefix.
#[test]
fn resolve_name_static_override() {
    // Arrange
    let source = test_source();
    let paths = build_path_manager(
        SharedOptions::mock(),
        NameOptions {
            name: Some("My Custom Name".to_owned()),
            ..NameOptions::default()
        },
    );

    // Act
    let dir = paths.get_transcode_target_dir(&source, TargetFormat::_320);

    // Assert
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, "My Custom Name [CD 320]");
}

/// Verify `--name-template` with `--experimental-name-template` renders metadata.
#[test]
fn resolve_name_template() {
    // Arrange
    let source = test_source();
    let paths = build_path_manager(
        SharedOptions::mock(),
        NameOptions {
            name_template: Some("{{ artist }} - {{ album }} [{{ year }}]".to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        },
    );

    // Act
    let dir = paths.get_transcode_target_dir(&source, TargetFormat::V0);

    // Assert
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, "Mock Artist - Test Album [2020]");
}

/// Verify template with conditional edition title.
#[test]
fn resolve_name_template_with_conditional() {
    // Arrange
    let source = test_source();
    let template = "{{ artist }} - {{ album }}{% if edition_title %} ({{ edition_title }}){% endif %} [{{ year }}]";
    let paths = build_path_manager(
        SharedOptions::mock(),
        NameOptions {
            name_template: Some(template.to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        },
    );

    // Act
    let dir = paths.get_spectrogram_dir(&source);

    // Assert
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, "Mock Artist - Test Album (Deluxe Edition) [2020]");
}

/// Verify template conditional omits empty edition title.
#[test]
fn resolve_name_template_without_edition() {
    // Arrange
    let mut source = test_source();
    source.metadata.edition_title = None;
    let template = "{{ artist }} - {{ album }}{% if edition_title %} ({{ edition_title }}){% endif %} [{{ year }}]";
    let paths = build_path_manager(
        SharedOptions::mock(),
        NameOptions {
            name_template: Some(template.to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        },
    );

    // Act
    let dir = paths.get_transcode_target_dir(&source, TargetFormat::Flac);

    // Assert
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, "Mock Artist - Test Album [2020]");
}

/// Verify `--name-template` can use the `{{ format }}` variable.
#[test]
fn resolve_name_template_with_format_variable() {
    // Arrange
    let source = test_source();
    let paths = build_path_manager(
        SharedOptions::mock(),
        NameOptions {
            name_template: Some("{{ artist }} - {{ album }} [{{ media }} {{ format }}]".to_owned()),
            experimental_name_template: true,
            ..NameOptions::default()
        },
    );

    // Act
    let dir = paths.get_transcode_target_dir(&source, TargetFormat::Flac);

    // Assert
    let dir_name = dir
        .file_name()
        .expect("should have file name")
        .to_str()
        .expect("should be valid utf-8");
    assert_eq!(dir_name, "Mock Artist - Test Album [CD FLAC]");
}
