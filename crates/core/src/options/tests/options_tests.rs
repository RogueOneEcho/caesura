use crate::testing_prelude::*;
use di::ServiceCollection;
use rogue_logging::{TimeFormat, Verbosity};
use std::fs::read_to_string;

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

/// Verify `CacheOptions` default uses platform user cache directory.
#[test]
fn cache_options_default_values() {
    let resolved = CacheOptionsPartial::default().resolve_without_validation();
    if is_docker() {
        assert_eq!(resolved.cache, PathBuf::from("/cache"));
    } else {
        assert!(
            resolved.cache.ends_with("caesura"),
            "expected cache path to end with 'caesura', got: {:?}",
            resolved.cache
        );
    }
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
        announce_url: Some(format!("{RED_TRACKER_URL}/abc123/announce")),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "red");
    assert_eq!(resolved.indexer_url, RED_URL);
}

/// Verify `indexer` and `indexer_url` are calculated from OPS `announce_url`.
#[test]
fn shared_options_calculates_indexer_from_ops_announce_url() {
    let resolved = SharedOptionsPartial {
        announce_url: Some(format!("{OPS_TRACKER_URL}/abc123/announce")),
        ..SharedOptionsPartial::default()
    }
    .resolve_without_validation();
    assert_eq!(resolved.indexer, "ops");
    assert_eq!(resolved.indexer_url, OPS_URL);
}

/// Verify explicit `indexer` is not overridden by calculated default.
#[test]
fn shared_options_does_not_override_explicit_indexer() {
    let resolved = SharedOptionsPartial {
        announce_url: Some(format!("{RED_TRACKER_URL}/abc123/announce")),
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
        announce_url: Some(format!("{RED_TRACKER_URL}/abc123/announce")),
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
/// The `required` attribute checks if the Option is None after `default_fn` runs.
/// An explicit `Some("")` is not None, so it passes the required check.
/// However, `indexer_url` still fails because its `default_fn` can't derive from "".
#[test]
fn shared_options_explicit_empty_indexer_bypasses_required() {
    let result = SharedOptionsPartial {
        announce_url: Some(format!("{RED_TRACKER_URL}/abc/announce")),
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
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::NotSet(name) if name == "Indexer url"))
    );
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

/// Verify explicitly empty `qbit_fetch_categories` list is rejected.
#[test]
fn queue_fetch_options_rejects_empty_categories_list() {
    let result = QueueFetchOptionsPartial {
        qbit_fetch_categories: Some(vec![]),
    }
    .resolve();
    let errors = result.expect_err("should reject empty categories list");
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
        allow_less_specific: Some(true),
        sox_random_dither: Some(true),
        exclude_vorbis_comments: Some(TargetOptions::default_exclude_vorbis_comments()),
    };

    // Act
    let yaml = serde_yaml::to_string(&original).expect("should serialize");
    let parsed: TargetOptionsPartial = serde_yaml::from_str(&yaml).expect("should deserialize");

    // Assert
    assert_eq!(original.target, parsed.target);
    assert_eq!(original.allow_existing, parsed.allow_existing);
    assert_eq!(original.sox_random_dither, parsed.sox_random_dither);
    assert_eq!(
        original.exclude_vorbis_comments,
        parsed.exclude_vorbis_comments
    );
}

/// Verify `SharedOptionsPartial` round-trips through YAML.
#[test]
fn shared_options_yaml_round_trip() {
    // Arrange
    let original = SharedOptionsPartial {
        announce_url: Some("https://example.com/announce".to_owned()),
        api_key: Some("secret_key".to_owned()),
        indexer: Some("red".to_owned()),
        indexer_url: Some(RED_URL.to_owned()),
        content: Some(vec![PathBuf::from("/data/music")]),
        output: Some(PathBuf::from("/data/output")),
        verbosity: Some(Verbosity::Debug),
        log_time: Some(TimeFormat::Elapsed),
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
    // Use explicit paths to avoid platform-specific and Docker-specific defaults in snapshot
    let result = SharedOptionsPartial {
        content: Some(vec![PathBuf::from("./nonexistent-content")]),
        output: Some(PathBuf::from("./nonexistent-output")),
        ..SharedOptionsPartial::default()
    }
    .resolve();
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
        indexer_url: Some(RED_URL.to_owned()),
        announce_url: Some(format!("{RED_TRACKER_URL}/abc/announce")),
        content: Some(vec![PathBuf::from(".")]),
        output: Some(PathBuf::from(".")),
        ..SharedOptionsPartial::default()
    }
    .resolve();
    assert!(result.is_ok());
}

/// Verify invalid hash format is rejected by `QueueRemoveArgs` validation.
#[test]
fn queue_rm_args_rejects_invalid_hash() {
    let result = QueueRemoveArgsPartial {
        queue_rm_hash: Some("not-a-valid-hash".to_owned()),
    }
    .resolve();
    let errors = result.expect_err("should reject invalid hash");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::HashInvalid(_, _)))
    );
}

/// Verify missing hash (defaults to empty string) is rejected.
#[test]
fn queue_rm_args_rejects_missing_hash() {
    let result = QueueRemoveArgsPartial {
        queue_rm_hash: None,
    }
    .resolve();
    let errors = result.expect_err("should reject missing hash");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::HashInvalid(_, _)))
    );
}

/// Verify valid 40-character hex hash is accepted.
#[test]
fn queue_rm_args_accepts_valid_hash() {
    let result = QueueRemoveArgsPartial {
        queue_rm_hash: Some("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_owned()),
    }
    .resolve();
    assert!(result.is_ok());
}

/// Verify literal `$HOME` in cache path fails validation.
#[test]
fn cache_options_rejects_dollar_home() {
    // Arrange
    let path = "$HOME/.cache/caesura";

    // Act
    let errors = CacheOptionsPartial {
        cache: Some(PathBuf::from(path)),
    }
    .resolve()
    .expect_err("should reject");

    // Assert
    assert_eq!(
        errors,
        vec![DoesNotExist(CACHE_DIR_LABEL.to_owned(), path.to_owned())]
    );
}

/// Verify `~` in cache path is expanded during validation.
///
/// Uses `~` alone because the home directory is guaranteed to exist.
/// Subpath expansion is covered by `expand_tilde` unit tests.
#[test]
fn cache_options_expands_tilde() {
    // Arrange
    let result = CacheOptionsPartial {
        cache: Some(PathBuf::from("~")),
    }
    .resolve();

    // Assert
    assert!(result.is_ok(), "~ should expand to home directory");
}

/// Verify literal `$HOME` in output path fails validation.
#[test]
fn shared_options_rejects_dollar_home_output() {
    // Arrange
    let path = "$HOME/.local/share/caesura/output";

    // Act
    let errors = valid_shared_options_with_output(path)
        .resolve()
        .expect_err("should reject");

    // Assert
    assert_eq!(
        errors,
        vec![DoesNotExist(OUTPUT_DIR_LABEL.to_owned(), path.to_owned())]
    );
}

/// Verify `~` in output path is expanded during validation.
///
/// Uses `~` alone because the home directory is guaranteed to exist.
#[test]
fn shared_options_expands_tilde_output() {
    // Arrange
    let result = valid_shared_options_with_output("~").resolve();

    // Assert
    assert!(result.is_ok(), "~ should expand to home directory");
}

/// Verify literal `$HOME` in content path fails validation.
#[test]
fn shared_options_rejects_dollar_home_content() {
    // Arrange
    let path = "$HOME/music";

    // Act
    let errors = valid_shared_options_with_content(path)
        .resolve()
        .expect_err("should reject");

    // Assert
    assert_eq!(
        errors,
        vec![DoesNotExist(CONTENT_DIR_LABEL.to_owned(), path.to_owned())]
    );
}

/// Verify `~` in content path is expanded during validation.
///
/// Uses `~` alone because the home directory is guaranteed to exist.
#[test]
fn shared_options_expands_tilde_content() {
    // Arrange
    let result = valid_shared_options_with_content("~").resolve();

    // Assert
    assert!(result.is_ok(), "~ should expand to home directory");
}

/// Verify `read_to_string` fails on literal `~` config path.
///
/// Config file resolution in `read_config_file` uses `read_to_string(path).ok()`
/// which silently swallows this error, meaning the config file is never loaded.
#[test]
fn config_read_to_string_fails_on_tilde() {
    assert!(read_to_string("~/.config/caesura/config.yml").is_err());
}

/// Verify `read_to_string` fails on literal `$HOME` config path.
#[test]
fn config_read_to_string_fails_on_dollar_home() {
    assert!(read_to_string("$HOME/.config/caesura/config.yml").is_err());
}

/// Verify literal `$HOME` in config path fails validation.
#[test]
fn config_options_rejects_dollar_home() {
    // Arrange
    let path = "$HOME/.config/caesura/config.yml";

    // Act
    let errors = ConfigOptionsPartial {
        config: Some(PathBuf::from(path)),
    }
    .resolve()
    .expect_err("should reject");

    // Assert
    assert_eq!(
        errors,
        vec![DoesNotExist(CONFIG_FILE_LABEL.to_owned(), path.to_owned())]
    );
}

/// Verify `~` in config path is expanded during validation.
///
/// The file won't exist, but the error path should show the expanded
/// absolute path rather than the literal `~`.
#[test]
fn config_options_expands_tilde() {
    // Arrange
    let partial = ConfigOptionsPartial {
        config: Some(PathBuf::from("~/.config/caesura/nonexistent.yml")),
    };

    // Act
    let errors = partial.resolve().expect_err("file should not exist");

    // Assert
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors.first(),
        Some(DoesNotExist(_, path)) if !path.starts_with('~')
    ));
}

fn valid_shared_options_with_output(output: &str) -> SharedOptionsPartial {
    SharedOptionsPartial {
        announce_url: Some(format!("{RED_TRACKER_URL}/abc/announce")),
        api_key: Some("key".to_owned()),
        content: Some(vec![PathBuf::from(".")]),
        output: Some(PathBuf::from(output)),
        ..SharedOptionsPartial::default()
    }
}

fn valid_shared_options_with_content(content: &str) -> SharedOptionsPartial {
    SharedOptionsPartial {
        announce_url: Some(format!("{RED_TRACKER_URL}/abc/announce")),
        api_key: Some("key".to_owned()),
        content: Some(vec![PathBuf::from(content)]),
        output: Some(PathBuf::from(".")),
        ..SharedOptionsPartial::default()
    }
}

/// Verify nonexistent path is rejected by `InspectArg` validation.
#[test]
fn inspect_arg_rejects_nonexistent_path() {
    let result = InspectArgPartial {
        inspect_path: Some(PathBuf::from("/nonexistent/path/that/does/not/exist")),
    }
    .resolve();
    let errors = result.expect_err("should reject nonexistent path");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::DoesNotExist(_, _)))
    );
}

/// Verify `QbitOptions::validate_connection` pushes `NotSet` errors for missing credentials.
#[test]
fn qbit_options_validate_connection_missing_credentials() {
    let options = QbitOptions {
        qbit_url: Some("http://127.0.0.1:8080".to_owned()),
        qbit_username: None,
        qbit_password: None,
    };
    let mut errors: Vec<OptionRule> = Vec::new();
    options.validate_connection(&mut errors);
    assert_eq!(
        errors,
        vec![
            OptionRule::NotSet("qBittorrent username".to_owned()),
            OptionRule::NotSet("qBittorrent password".to_owned())
        ]
    );
}

/// Verify `QbitOptions::validate_connection` does not require credentials for a qui proxy URL.
#[test]
fn qbit_options_validate_connection_qui_proxy_url() {
    let options = QbitOptions {
        qbit_url: Some("http://localhost:7476/proxy/abc123".to_owned()),
        qbit_username: None,
        qbit_password: None,
    };
    let mut errors: Vec<OptionRule> = Vec::new();
    options.validate_connection(&mut errors);
    assert!(errors.is_empty());
}

/// Verify `QbitOptions` rejects a trailing slash in the URL.
#[test]
fn qbit_options_trailing_slash() {
    let mock = QbitOptions::mock();
    let partial = QbitOptionsPartial {
        qbit_url: Some("http://localhost:7476/proxy/abc123/".to_owned()),
        qbit_username: mock.qbit_username,
        qbit_password: mock.qbit_password,
    };
    let result = partial.resolve();
    let errors = result.expect_err("should reject trailing slash");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, OptionRule::UrlInvalidSuffix(_, _)))
    );
}

/// Verify invalid YAML produces a config deserialization error.
#[test]
#[expect(non_snake_case, reason = "double underscore test qualifier convention")]
fn options_provider_register__invalid_yaml() {
    // Arrange
    let yaml = "qbit_url:\n  - not\n  - a\n  - string\n".to_owned();
    let mut provider = OptionsProvider::from_yaml(Some(yaml));
    let mut services = ServiceCollection::new();

    // Act
    provider.register::<QbitOptionsPartial>(&mut services);

    // Assert
    assert!(provider.has_errors());
    let error = provider.errors.first().expect("should have an error");
    assert!(
        matches!(error, OptionRule::ConfigDeserialize(_)),
        "Expected ConfigDeserialize, got: {error}"
    );
}

/// Verify valid YAML does not produce errors.
#[test]
#[expect(non_snake_case, reason = "double underscore test qualifier convention")]
fn options_provider_register__valid_yaml() {
    // Arrange
    let yaml = "qbit_url: http://127.0.0.1:8080\n".to_owned();
    let mut provider = OptionsProvider::from_yaml(Some(yaml));
    let mut services = di::ServiceCollection::new();

    // Act
    provider.register::<QbitOptionsPartial>(&mut services);

    // Assert
    assert!(!provider.has_errors());
}

/// Verify missing YAML does not produce errors.
#[test]
#[expect(non_snake_case, reason = "double underscore test qualifier convention")]
fn options_provider_register__no_yaml() {
    // Arrange
    let mut provider = OptionsProvider::from_yaml(None);
    let mut services = di::ServiceCollection::new();

    // Act
    provider.register::<QbitOptionsPartial>(&mut services);

    // Assert
    assert!(!provider.has_errors());
}
