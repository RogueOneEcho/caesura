use crate::prelude::*;
use std::fs::canonicalize;

/// Options for the `audit` command.
#[derive(Options, Clone, Debug, Deserialize, Serialize)]
pub struct AuditArgs {
    /// A tracker torrent id, a single `.torrent` file, or a directory of `.torrent` files.
    ///
    /// Examples:
    /// - `12345`
    /// - `/srv/qBittorrent/BT_backup`
    /// - `path/to/file.torrent`
    #[arg(value_name = "PATH | ID")]
    pub audit_arg: String,
}

/// Input to the `audit` command.
pub enum AuditMode {
    /// A tracker torrent id to download and audit.
    Id(u32),
    /// A torrent file path.
    File(PathBuf),
    /// A directory path.
    Directory(PathBuf),
}

impl AuditArgs {
    /// Classify [`AuditArgs::input`] as a [`AuditMode`]
    pub fn to_mode(&self) -> Result<AuditMode, OptionIssue> {
        if let Ok(id) = self.audit_arg.parse::<u32>() {
            return Ok(AuditMode::Id(id));
        }
        let path = PathBuf::from(&self.audit_arg);
        if !path.exists() {
            return Err(OptionIssue::path_not_found("audit_arg", &path));
        }
        let path = canonicalize(path).expect("should be able to canonicalize path");
        if path.is_file() {
            Ok(AuditMode::File(path))
        } else if path.is_dir() {
            Ok(AuditMode::Directory(path))
        } else {
            Err(OptionIssue::path_not_found("audit_arg", &path))
        }
    }
}

impl OptionsContract for AuditArgs {
    type Partial = AuditArgsPartial;

    fn validate(&self, validator: &mut OptionsValidator) {
        if let Err(issue) = self.to_mode() {
            validator.push(issue);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;

    #[test]
    fn audit_args_to_mode_id() {
        let args = AuditArgs {
            audit_arg: "12345".to_owned(),
        };
        assert!(matches!(args.to_mode(), Ok(AuditMode::Id(12345))));
    }

    /// An existing file path resolves to [`AuditMode::File`].
    #[test]
    fn audit_args_to_mode_file() {
        // Arrange
        let dir = TempDirectory::create("audit_args_to_mode_file");
        let file = dir.join("file.torrent");
        write(&file, b"data").expect("write file");
        let args = AuditArgs {
            audit_arg: file.to_string_lossy().into_owned(),
        };
        // Act
        let mode = args.to_mode();
        // Assert
        assert!(matches!(mode, Ok(AuditMode::File(_))));
    }

    /// An existing directory path resolves to [`AuditMode::Directory`].
    #[test]
    fn audit_args_to_mode_directory() {
        // Arrange
        let dir = TempDirectory::create("audit_args_to_mode_directory");
        let args = AuditArgs {
            audit_arg: dir.to_string_lossy().into_owned(),
        };
        // Act
        let mode = args.to_mode();
        // Assert
        assert!(matches!(mode, Ok(AuditMode::Directory(_))));
    }

    /// A numeric id skips the path-exists check even though no such file exists.
    #[test]
    fn audit_args_validate_id() {
        let args = AuditArgs {
            audit_arg: "12345".to_owned(),
        };
        let mut validator = OptionsValidator::new();
        args.validate(&mut validator);
        assert!(validator.into_issues().is_empty());
    }

    /// A non-numeric input pointing at a missing path reports one issue.
    #[test]
    fn audit_args_validate_path() {
        let args = AuditArgs {
            audit_arg: "/no/such/path/at/all".to_owned(),
        };
        let mut validator = OptionsValidator::new();
        args.validate(&mut validator);
        let issues = validator.into_issues();
        assert_eq!(issues.len(), 1);
    }
}
