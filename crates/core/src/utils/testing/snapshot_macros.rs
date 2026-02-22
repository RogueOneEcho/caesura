//! Snapshot macros for deterministic testing across platforms.

/// Resolve the path to the stored `.snap` file for the calling test function.
macro_rules! resolve_snap_path {
    () => {{
        let workspace = insta::_get_workspace_root!();
        let function_name = insta::_function_name!();
        let snap_name = function_name
            .rsplit("::")
            .next()
            .expect("function name should have segments");
        let module_prefix = module_path!().replace("::", "__");
        let snap_filename = format!("{module_prefix}__{snap_name}.snap");
        workspace
            .join(
                std::path::Path::new(file!())
                    .parent()
                    .expect("source file should have parent directory"),
            )
            .join("snapshots")
            .join(&snap_filename)
    }};
}

/// Normalize non-deterministic fields in a [`FileSnapshot`] vec for snapshot comparison.
///
/// When `CAESURA_DETERMINISTIC_TESTS` is set, returns the snapshot unchanged for
/// exact matching. Otherwise, patches SHA-256 and file-size fields from the
/// stored `.snap` file so only structural differences cause failures.
macro_rules! normalize_snapshots {
    ($snapshot:expr) => {
        if is_deterministic() {
            $snapshot
        } else {
            let mut files = $snapshot;
            let snap_path = resolve_snap_path!();
            crate::utils::patch_platform_dependent_fields(&mut files, &snap_path);
            files
        }
    };
}

/// Assert a string snapshot, or verify line count when deterministic tests are disabled.
///
/// When `CAESURA_DETERMINISTIC_TESTS` is set, performs an exact snapshot match.
/// Otherwise, verifies the output has the same number of lines as the stored
/// snapshot.
macro_rules! assert_inspect_snapshot {
    ($output:expr) => {
        if is_deterministic() {
            insta::assert_snapshot!($output);
        } else {
            let snap_path = resolve_snap_path!();
            crate::utils::assert_line_count(&$output, &snap_path);
        }
    };
}

pub(crate) use assert_inspect_snapshot;
pub(crate) use normalize_snapshots;
pub(crate) use resolve_snap_path;
