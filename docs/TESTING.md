# Testing

## Overview

This project has comprehensive unit and integration tests.

Many of these are deterministic which relies on you having the same dependency versions as the CI runner.

Dependencies can differ between Intel/AMD (`amd64`) and ARM (`arm64`) CPU architecture. Therefore it is recommended to test on Intel/AMD as many tests are marked as ignored on ARM.

## Required Tools

The following tools must be installed to run tests. Version numbers are those used during development - other versions may work but could produce different output for snapshot tests.

| Tool        | Version  | Purpose                             | Installation              |
|-------------|----------|-------------------------------------|---------------------------|
| SoX_ng      | 14.7.1   | Generate sample audio files         | `brew install sox_ng`     |
| FLAC        | 1.5.0    | FLAC encoding/decoding              | `brew install flac`       |
| metaflac    | 1.5.0    | Test sample tag injection           | Included with FLAC        |
| LAME        | 3.100    | MP3 encoding for transcode tests    | `brew install lame`       |

## Running Tests

```bash
cargo test --quiet --all-features
```

This runs all non-ignored tests, which use:
- Generated sample FLAC files (created on first run)
- Mock API client (no network access required)
- Snapshot testing with `insta`

Tests should complete quickly even with a fresh sample cache. Expect ~45s on an `ubuntu-24.04` CI runner and under 10s on a modern desktop. Significantly longer could indicate an issue with a dependency.

```
$ ./samples/rm-samples && cargo test --quiet --all-features
test result: ok. 249 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 8.70s
```

## Removing samples

The test infrastructure generates and caches sample FLAC files programmatically ensuring reproducible and self-contained tests.

Occasionally these can become stale or corrupted. Particularly if there have been many upstream changes. There if tests are failing unexpectly you may wish to clear the samples and force them to regenerate with this little utility:

```bash
./samples/rm-samples
```

## Running Determinism Tests

To verify sample generation produces consistent output:

```bash
cargo test --release -- --ignored sample_flac
```

These tests generate samples in temporary directories and compare against stored snapshots.

## Test Infrastructure

### Sample Generation (`crates/core/src/utils/testing/samples/`)

- **`FlacGenerator`** - Builder for creating FLAC files with specific parameters
- **`ImageGenerator`** - Creates deterministic PNG images for cover art
- **`SampleDataBuilder`** - Orchestrates full sample set (FLAC + cover + torrent)
- **`SampleFormat`** - Type-safe audio format specifications

Sample files are cached in `samples/content/` with the following naming convention:
```
{Artist} - {Album} ({Year}) [WEB] {bit-depth}-{sample-rate} (FLAC)/
```

### Snapshot Testing (`crates/core/src/utils/testing/snapshots/`)

Uses the `insta` crate for snapshot testing:

- **`AudioSnapshot`** - Captures audio metadata, tags, and embedded pictures
- **`ImageSnapshot`** - Captures image dimensions and color type
- **`FileSnapshot`** - File metadata with SHA-256 hash
- **`DirectorySnapshot`** - Aggregates file snapshots for a directory

### Test Utilities (`crates/core/src/utils/testing/`)

- **`TempDirectory::create(name)`** - Creates isolated temp directories with timestamp-based uniqueness
- **`TestDirectory::new()`** - Creates test directory structure with `output` and `cache` subdirectories
- **`AlbumConfig::single_torrent_dir()`** - Copies a single torrent to an isolated temp directory (use this instead of `SAMPLE_SOURCES_DIR` for tests that need a specific torrent)
- **`AlbumProvider::get(format)`** - Gets or generates sample album configuration
- **`TranscodeProvider::get(source, target)`** - Gets or generates a cached transcode

### Mock API Client

The `gazelle_api` crate provides `MockGazelleClient` for testing without network access.

Use `AlbumConfig::api()` for a pre-configured mock, or build one manually with `MockGazelleClient::new()` and pass it to `HostBuilder` via `with_mock_client()`.

### Host Setup

Most integration tests build a DI container with `HostBuilder`:

- **`HostBuilder::new()`** - Creates an empty DI container
- **`.with_mock_api(album)`** - Registers a mock Gazelle client from an `AlbumConfig`
- **`.with_test_options(&test_dir)`** - Registers `SharedOptions` and `CacheOptions` pointing at test directories
- **`.with_options(T)`** - Registers additional option structs (e.g. `UploadOptions`, `QueueAddArgs`)
- **`.expect_build()`** - Builds the container, panicking on error
- **`host.services.get_required::<T>()`** - Resolves a service from the container

Typical setup:

```rust
#[tokio::test]
async fn example() -> Result<(), TestError> {
    // Arrange
    init_logger();
    let album = AlbumProvider::get(SampleFormat::default()).await;
    let test_dir = TestDirectory::new();
    let host = HostBuilder::new()
        .with_mock_api(album)
        .with_test_options(&test_dir)
        .await
        .with_options(UploadOptions { dry_run: true, ..UploadOptions::default() })
        .expect_build();
    let command = host.services.get_required::<UploadCommand>();

    // Act
    let result = command.execute_cli().await;

    // Assert
    assert!(result.is_ok());
    Ok(())
}
```

> [!WARNING]
> `TestDirectory` is cleaned up on drop. Bind it to a named variable (`test_dir` or `_test_dir`), not `_`, or the directory will be deleted mid-test.

## Deterministic Snapshot Tests

Some snapshot tests verify the deterministic output via SHA-256 hashes and file sizes. However, this is not guaranteed to be identical across platforms:

- libFLAC does not guarantee identical output [across machines or builds](https://xiph.org/flac/faq.html#tools__different_sizes)
- SoX is built against libFLAC, so differences in either the SoX or FLAC build can produce different spectrogram images even at the same version.

Where cross platform behavior is non-deterministic we use x86_64 Linux as the source of truth and ignore the deterministic aspects on other platforms.

To enable full deterministic set the `CAESURA_DETERMINISTIC_TESTS` environment variable:

```bash
CAESURA_DETERMINISTIC_TESTS=1 cargo test
```

| Test                         | Guard                      | Non-deterministic behavior                                                                    |
|------------------------------|----------------------------|-----------------------------------------------------------------------------------------------|
| `transcode_command_*`        | `normalize_snapshots!`     | SHA-256, file-size, and bitrate ignored                                                                 |
| `transcode_rename_tracks_*` | `normalize_snapshots!`     | SHA-256, file-size, and bitrate ignored                                                                 |
| `inspect_*`                  | `assert_inspect_snapshot!` | Line count only                                                                               |
| `spectrogram_command_*`      | `normalize_snapshots!`     | SHA-256, file-size, and bitrate ignored                                                                 |
| `sample_flac*`               | `#[ignore]`                | Ignored, CI only runs on x86_64 Linux via<br>`cargo test --release -- --ignored sample_flac` |

### Updating Snapshots

You may need to [install cargo-insta](https://insta.rs/docs/quickstart/#installation).

To run all tests and review the snapshot diffs:

```bash
cargo insta test --review
```

To accept all diffs:

```bash
cargo insta accept
```

### Macro Expansion Snapshots

The `caesura_macros` crate has snapshot tests that expand each options struct via the `Options` derive macro (e.g. `expand_shared_options`, `expand_target_options`). These snapshots must be updated whenever a file in `crates/core/src/options/` is modified — including field additions, removals, renames, doc comment changes, or attribute changes.

```bash
cargo insta test -p caesura_macros
```
