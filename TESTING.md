# Testing

This document describes the testing infrastructure for Caesura.

## Overview

Tests are divided into two categories:

1. **Unit/Integration tests** - Run by default with `cargo test`
2. **Live API tests** - Marked with `#[ignore]`, require real API credentials

The test infrastructure generates sample FLAC files programmatically rather than downloading external files, ensuring reproducible and self-contained tests.

## Required Tools

The following tools must be installed to run tests. Version numbers are those used during development - other versions may work but could produce different output for snapshot tests.


| Tool        | Version | Purpose                             | Installation              |
|-------------|---------|-------------------------------------|---------------------------|
| SoX         | 14.4.2  | Generate sample audio files         | `brew install sox`        |
| FLAC        | 1.5.0   | FLAC encoding/decoding              | `brew install flac`       |
| metaflac    | 1.5.0   | FLAC metadata manipulation          | Included with FLAC        |
| LAME        | 3.100   | MP3 encoding for transcode tests    | `brew install lame`       |
| imdl        | 0.1.13  | Torrent file creation               | `brew install intermodal` |

## Running Tests

### Standard Tests

```bash
cargo test
```

This runs all non-ignored tests, which use:
- Generated sample FLAC files (created on first run)
- Mock API client (no network access required)
- Snapshot testing with `insta`

### Determinism Tests

To verify sample generation produces consistent output:

```bash
cargo test --release -- --ignored sample_flac
```

These tests generate samples in temporary directories and compare against stored snapshots.

### Live API Tests

Tests marked with `#[ignore = "Integration test requiring live API"]` require:
- A valid `config.yml` with API credentials
- Network access to the tracker API

```bash
cargo test --release -- --ignored live
```

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

### Mock API Client

The `gazelle_api` crate provides `MockGazelleClient` for testing without network access. Configure via `HostBuilder`:

## Determinism Considerations

For reproducible test output:

1. **SoX** is invoked with `-D` flag to disable dithering
2. **24-bit transcode tests** are marked as ignored because SoX dithering during bit-depth conversion is non-deterministic
3. **Torrent files** are excluded from snapshots (contain timestamps)
4. **FLAC encoder version** is embedded in files - snapshots may need updating when FLAC is upgraded

### Platform-Specific Behavior

Snapshot tests verifying binary output (SHA256 hashes) only pass on x86_64. On ARM (aarch64), libFLAC produces different output due to NEON SIMD optimizations using different floating-point precision than SSE. This is [documented and expected](https://xiph.org/flac/faq.html).

| Test                         | x86_64    | aarch64   |
|------------------------------|-----------|-----------|
| `transcode_command_flac16_*` | ✅        | ❌ skipped |
| `sample_flac*` (16-bit)      | ✅        | ❌ skipped |
| `sample_flac*` (24-bit)      | ❌ skipped | ❌ skipped |

24-bit sample tests are skipped on all macOS due to additional floating-point sensitivity in LPC prediction at higher bit depths.

### Updating Snapshots

If tool versions change and produce different (but valid) output:

```bash
cargo insta test --review
```
