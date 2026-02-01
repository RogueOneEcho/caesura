use crate::testing_prelude::*;
use rogue_logging::{TimeFormat, Verbosity};

/// Verify `BatchOptions` default values.
#[test]
fn batch_options_default_values() {
    let result = BatchOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `FileOptions` default values.
#[test]
fn file_options_default_values() {
    let result = FileOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `TargetOptions` default values.
#[test]
fn target_options_default_values() {
    let result = TargetOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `SpectrogramOptions` default values.
#[test]
fn spectrogram_options_default_values() {
    let result = SpectrogramOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `UploadOptions` default values.
#[test]
fn upload_options_default_values() {
    let result = UploadOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `VerifyOptions` default values.
#[test]
fn verify_options_default_values() {
    let result = VerifyOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `CacheOptions` default values.
#[test]
fn cache_options_default_values() {
    let default = CacheOptions::default();
    assert_yaml_snapshot!(default);
}

/// Verify `CopyOptions` default values.
#[test]
fn copy_options_default_values() {
    let result = CopyOptionsPartial::default().resolve_without_validation();
    assert_yaml_snapshot!(result);
}

/// Verify `indexer` and `indexer_url` are calculated from RED `announce_url`.
#[test]
fn shared_options_calculates_indexer_from_red_announce_url() {
    let resolved = SharedOptionsPartial {
        announce_url: Some("https://flacsfor.me/abc123/announce".to_owned()),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "red");
    assert_eq!(resolved.indexer_url, "https://redacted.sh");
}

/// Verify `indexer` and `indexer_url` are calculated from OPS `announce_url`.
#[test]
fn shared_options_calculates_indexer_from_ops_announce_url() {
    let resolved = SharedOptionsPartial {
        announce_url: Some("https://home.opsfet.ch/abc123/announce".to_owned()),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "ops");
    assert_eq!(resolved.indexer_url, "https://orpheus.network");
}

/// Verify explicit `indexer` is not overridden by calculated default.
#[test]
fn shared_options_does_not_override_explicit_indexer() {
    let resolved = SharedOptionsPartial {
        announce_url: Some("https://flacsfor.me/abc123/announce".to_owned()),
        indexer: Some("custom".to_owned()),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "custom");
    // indexer_url uses the custom indexer which doesn't match red/ops, so defaults to empty
    assert_eq!(resolved.indexer_url, "");
}

/// Verify explicit `indexer_url` is not overridden by calculated default.
#[test]
fn shared_options_does_not_override_explicit_indexer_url() {
    let resolved = SharedOptionsPartial {
        announce_url: Some("https://flacsfor.me/abc123/announce".to_owned()),
        indexer_url: Some("https://custom.example.com".to_owned()),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "red");
    assert_eq!(resolved.indexer_url, "https://custom.example.com");
}

/// Verify unknown `announce_url` leaves `indexer` and `indexer_url` as empty (fails validation).
#[test]
fn shared_options_unknown_announce_url_leaves_indexer_empty() {
    let resolved = SharedOptionsPartial {
        announce_url: Some("https://unknown.tracker.com/announce".to_owned()),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    // With resolve_without_validation, required fields default to empty string
    assert_eq!(resolved.indexer, "");
    assert_eq!(resolved.indexer_url, "");
}

/// Edge case: explicit empty string for `indexer` bypasses `required` check.
///
/// The `required` attribute checks if the Option is None after default_fn runs.
/// An explicit `Some("")` is not None, so it passes the required check.
/// However, `indexer_url` still fails because its default_fn can't derive from "".
#[test]
fn shared_options_explicit_empty_indexer_bypasses_required() {
    let result = SharedOptionsPartial {
        announce_url: Some("https://flacsfor.me/abc/announce".to_owned()),
        api_key: Some("key".to_owned()),
        indexer: Some(String::new()), // Explicit empty string
        content: Some(vec![PathBuf::from(".")]),
        output: Some(PathBuf::from(".")),
        ..SharedOptionsPartial::default()
    }
    .resolve();
    // indexer passes required check (Some("") is not None)
    // but indexer_url fails because default_fn can't derive from empty indexer
    let errors = result.expect_err("should fail due to indexer_url");
    assert!(errors.iter().any(|e| matches!(e, OptionRule::NotSet(name) if name == "Indexer url")));
}

/// Verify `upload` without `transcode` is rejected.
#[test]
fn batch_options_rejects_upload_without_transcode() {
    let result = BatchOptionsPartial {
        upload: Some(true),
        transcode: None,
        ..BatchOptionsPartial::default()
    }
    .resolve();
    let errors = result.expect_err("should reject upload without transcode");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::Dependent(_, _)))
    );
}

/// Verify `upload` with `transcode` is accepted.
#[test]
fn batch_options_accepts_upload_with_transcode() {
    let result = BatchOptionsPartial {
        upload: Some(true),
        transcode: Some(true),
        ..BatchOptionsPartial::default()
    }
    .resolve();
    assert!(result.is_ok());
}

/// Verify invalid `wait_before_upload` duration is rejected.
#[test]
fn batch_options_rejects_invalid_wait_duration() {
    let result = BatchOptionsPartial {
        wait_before_upload: Some("invalid".to_owned()),
        ..BatchOptionsPartial::default()
    }
    .resolve();
    let errors = result.expect_err("should reject invalid duration");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::DurationInvalid(_, _)))
    );
}

/// Verify valid `wait_before_upload` duration is accepted.
#[test]
fn batch_options_accepts_valid_wait_duration() {
    let result = BatchOptionsPartial {
        wait_before_upload: Some("5m30s".to_owned()),
        ..BatchOptionsPartial::default()
    }
    .resolve();
    assert!(result.is_ok());
}

/// Verify explicitly empty `target` list is rejected.
#[test]
fn target_options_rejects_empty_target_list() {
    let result = TargetOptionsPartial {
        target: Some(vec![]),
        ..TargetOptionsPartial::default()
    }
    .resolve();
    let errors = result.expect_err("should reject empty target list");
    assert!(errors.iter().any(|e| matches!(e, OptionRule::IsEmpty(_))));
}

/// Verify explicitly empty `spectrogram_size` list is rejected.
#[test]
fn spectrogram_options_rejects_empty_size_list() {
    let result = SpectrogramOptionsPartial {
        spectrogram_size: Some(vec![]),
    }
    .resolve();
    let errors = result.expect_err("should reject empty size list");
    assert!(errors.iter().any(|e| matches!(e, OptionRule::IsEmpty(_))));
}

/// Verify `BatchOptionsPartial` round-trips through YAML.
#[test]
fn batch_options_yaml_round_trip() {
    // Arrange
    let original = BatchOptionsPartial {
        spectrogram: Some(true),
        transcode: Some(true),
        upload: Some(true),
        limit: Some(5),
        no_limit: Some(false),
        wait_before_upload: Some("10m".to_owned()),
        ..BatchOptionsPartial::default()
    };

    // Act
    let yaml = serde_yaml::to_string(&original).expect("should serialize");
    let parsed: BatchOptionsPartial = serde_yaml::from_str(&yaml).expect("should deserialize");

    // Assert
    assert_eq!(original.spectrogram, parsed.spectrogram);
    assert_eq!(original.transcode, parsed.transcode);
    assert_eq!(original.upload, parsed.upload);
    assert_eq!(original.limit, parsed.limit);
    assert_eq!(original.wait_before_upload, parsed.wait_before_upload);
}

/// Verify `TargetOptionsPartial` round-trips through YAML.
#[test]
fn target_options_yaml_round_trip() {
    // Arrange
    let original = TargetOptionsPartial {
        target: Some(vec![TargetFormat::Flac, TargetFormat::V0]),
        allow_existing: Some(true),
        sox_random_dither: Some(true),
    };

    // Act
    let yaml = serde_yaml::to_string(&original).expect("should serialize");
    let parsed: TargetOptionsPartial = serde_yaml::from_str(&yaml).expect("should deserialize");

    // Assert
    assert_eq!(original.target, parsed.target);
    assert_eq!(original.allow_existing, parsed.allow_existing);
    assert_eq!(original.sox_random_dither, parsed.sox_random_dither);
}

/// Verify `SharedOptionsPartial` round-trips through YAML.
#[test]
fn shared_options_yaml_round_trip() {
    // Arrange
    let original = SharedOptionsPartial {
        announce_url: Some("https://example.com/announce".to_owned()),
        api_key: Some("secret_key".to_owned()),
        indexer: Some("red".to_owned()),
        indexer_url: Some("https://redacted.sh".to_owned()),
        content: Some(vec![PathBuf::from("/data/music")]),
        output: Some(PathBuf::from("/data/output")),
        verbosity: Some(Verbosity::Debug),
        log_time: Some(TimeFormat::Elapsed),
        config: Some(PathBuf::from("./custom_config.yml")),
    };

    // Act
    let yaml = serde_yaml::to_string(&original).expect("should serialize");
    let parsed: SharedOptionsPartial = serde_yaml::from_str(&yaml).expect("should deserialize");

    // Assert
    assert_eq!(original.announce_url, parsed.announce_url);
    assert_eq!(original.api_key, parsed.api_key);
    assert_eq!(original.indexer, parsed.indexer);
    assert_eq!(original.content, parsed.content);
    assert_eq!(original.verbosity, parsed.verbosity);
}

/// Verify CLI values override YAML values during merge.
#[test]
fn merge_cli_overrides_yaml() {
    // Arrange
    let mut cli = BatchOptionsPartial {
        limit: Some(10),
        ..BatchOptionsPartial::default()
    };
    let yaml = BatchOptionsPartial {
        limit: Some(5),
        spectrogram: Some(true),
        transcode: Some(true),
        ..BatchOptionsPartial::default()
    };

    // Act
    cli.merge(yaml);

    // Assert
    assert_eq!(cli.limit, Some(10));
    assert_eq!(cli.spectrogram, Some(true));
    assert_eq!(cli.transcode, Some(true));
}

/// Verify YAML values fill in None values during merge.
#[test]
fn merge_yaml_fills_none_values() {
    // Arrange
    let mut cli = BatchOptionsPartial::default();
    let yaml = BatchOptionsPartial {
        limit: Some(5),
        spectrogram: Some(true),
        ..BatchOptionsPartial::default()
    };

    // Act
    cli.merge(yaml);

    // Assert
    assert_eq!(cli.limit, Some(5));
    assert_eq!(cli.spectrogram, Some(true));
}

/// Verify merge does not override already-set values.
#[test]
fn merge_does_not_override_set_values() {
    // Arrange
    let mut cli = SharedOptionsPartial {
        indexer: Some("custom".to_owned()),
        verbosity: Some(Verbosity::Trace),
        ..SharedOptionsPartial::default()
    };
    let yaml = SharedOptionsPartial {
        indexer: Some("red".to_owned()),
        verbosity: Some(Verbosity::Info),
        api_key: Some("from_yaml".to_owned()),
        ..SharedOptionsPartial::default()
    };

    // Act
    cli.merge(yaml);

    // Assert
    assert_eq!(cli.indexer, Some("custom".to_owned()));
    assert_eq!(cli.verbosity, Some(Verbosity::Trace));
    assert_eq!(cli.api_key, Some("from_yaml".to_owned()));
}

/// Verify `with_options` overrides options registered by `register_options`.
#[test]
fn with_options_overrides_default_registration() {
    use crate::hosting::HostBuilder;

    // Arrange: HostBuilder::new() registers default options via register_options()
    let mut builder = HostBuilder::new();

    // Act: Override with custom value
    let custom_output = PathBuf::from("/custom/output/path");
    let host = builder
        .with_options(SharedOptions {
            output: custom_output.clone(),
            ..SharedOptions::default()
        })
        .expect_build();

    // Assert: Retrieved options should have the overridden value
    let options = host.services.get_required::<SharedOptions>();
    assert_eq!(options.output, custom_output);
}

/// Verify validation errors for missing required fields.
#[test]
fn shared_options_validate_missing_fields() {
    let result = SharedOptionsPartial::default().resolve();
    let errors = result.expect_err("should reject missing required fields");
    assert_yaml_snapshot!(errors);
}

/// Verify validation errors for invalid URLs.
#[test]
fn shared_options_validate_invalid_urls() {
    let result = SharedOptionsPartial {
        api_key: Some("key".to_owned()),
        indexer: Some("red".to_owned()),
        indexer_url: Some("not-a-url".to_owned()),
        announce_url: Some("https://example.com/announce/".to_owned()),
        content: Some(vec![PathBuf::from(".")]),
        output: Some(PathBuf::from(".")),
        ..SharedOptionsPartial::default()
    }
    .resolve();
    let errors = result.expect_err("should reject invalid URLs");
    assert_yaml_snapshot!(errors);
}

/// Verify valid options produce no validation errors.
#[test]
fn shared_options_validate_no_errors_when_valid() {
    let result = SharedOptionsPartial {
        api_key: Some("key".to_owned()),
        indexer: Some("red".to_owned()),
        indexer_url: Some("https://redacted.sh".to_owned()),
        announce_url: Some("https://flacsfor.me/abc/announce".to_owned()),
        content: Some(vec![PathBuf::from(".")]),
        output: Some(PathBuf::from(".")),
        ..SharedOptionsPartial::default()
    }
    .resolve();
    assert!(result.is_ok());
}
