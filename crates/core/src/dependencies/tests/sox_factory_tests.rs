use std::path::PathBuf;
use std::sync::Arc;

use crate::dependencies::SoxFactory;
use crate::testing_prelude::*;

/// When only `sox_ng: true` is set, the factory selects the `sox_ng` binary.
#[test]
fn sox_factory_sox_ng_true() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_ng: Some(true),
        sox_path: None,
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), SOX_NG);
    assert!(
        factory
            .create()
            .args
            .contains(&"--single-threaded".to_owned())
    );
}

/// When only `sox_ng: false` is set, the factory selects the `sox` binary.
#[test]
fn sox_factory_sox_ng_false() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_ng: Some(false),
        sox_path: None,
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), SOX);
    assert!(factory.create().args.is_empty());
}

/// When `sox_path` contains `sox_ng`, `sox_ng` defaults to `true`.
#[test]
fn sox_factory_path_containing_sox_ng() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_path: Some(PathBuf::from("/usr/local/bin/sox_ng")),
        sox_ng: None,
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), "/usr/local/bin/sox_ng");
    assert!(
        factory
            .create()
            .args
            .contains(&"--single-threaded".to_owned())
    );
}

/// When `sox_path` does not contain `sox_ng`, `sox_ng` falls back to auto-detection.
#[test]
fn sox_factory_path_without_sox_ng() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_path: Some(PathBuf::from("/usr/local/bin/sox")),
        sox_ng: None,
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), "/usr/local/bin/sox");
}

/// When both are set explicitly, no defaulting occurs.
#[test]
fn sox_factory_both_explicit() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_path: Some(PathBuf::from("/opt/sox/bin/sox")),
        sox_ng: Some(true),
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), "/opt/sox/bin/sox");
    assert!(
        factory
            .create()
            .args
            .contains(&"--single-threaded".to_owned())
    );
}

/// When both are set explicitly with `sox_ng: false`, the path is preserved as-is.
#[test]
fn sox_factory_explicit_path_sox_ng_false() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_path: Some(PathBuf::from("/custom/sox_ng")),
        sox_ng: Some(false),
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    assert_eq!(factory.binary(), "/custom/sox_ng");
    assert!(
        factory.create().args.is_empty(),
        "explicit sox_ng: false should not be overridden by path name"
    );
}

/// When neither is set, the factory falls back to auto-detected behavior.
#[test]
fn sox_factory_neither_set() {
    // Arrange
    let options = SoxOptionsPartial {
        sox_path: None,
        sox_ng: None,
    };
    let factory = factory(options.resolve_without_validation());

    // Act / Assert
    // sox_ng is auto-detected — just verify binary() returns a valid name without panicking
    let binary = factory.binary();
    assert!(binary == SOX || binary == SOX_NG);
}

fn factory(options: SoxOptions) -> SoxFactory {
    SoxFactory::new(Arc::new(options))
}
