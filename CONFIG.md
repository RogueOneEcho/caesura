# Configuration Reference

This document describes all configuration options available in caesura.
Options can be set via CLI flags or in `config.yml`.

## Options shared by all commands

**Applies to:** `Batch`, `Queue`, `Spectrogram`, `Transcode`, `Verify`, `Upload`

| Config Key     | CLI Flag         | Type              | Default                            | Description                                                                                                                           |
|----------------|------------------|-------------------|------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| `announce_url` | `--announce-url` | `Option<String>`  | -                                  | Announce URL including passkey Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`  |
| `api_key`      | `--api-key`      | `Option<String>`  | -                                  | API key with torrent permissions for the indexer.                                                                                     |
| `indexer`      | `--indexer`      | `Option<String>`  | -                                  | ID of the tracker as it appears in the source field of a torrent. Examples: `red`, `pth`, `ops` Default: Determined by `announce_url` |
| `indexer_url`  | `--indexer-url`  | `Option<String>`  | -                                  | URL of the indexer. Examples: `https://redacted.sh`, `https://orpheus.network` Default: Determined by `announce_url`                  |
| `content`      | `--content`      | `Vec<PathBuf>`    | `vec![PathBuf::from("./content")]` | Directories containing torrent content. Typically this is set as the download directory in your torrent client. Default: `./content`  |
| `verbosity`    | `--verbosity`    | `Verbosity`       | -                                  | Level of logs to display. Default: `info`                                                                                             |
| `config`       | `--config`       | `Option<PathBuf>` | -                                  | Path to the configuration file. Default: `./config.yml`                                                                               |
| `log_time`     | `--log-time`     | `TimeFormat`      | -                                  | Time format to use in logs. Default: `Local`                                                                                          |
| `output`       | `--output`       | `PathBuf`         | `PathBuf::from("./output")`        | Directory where transcodes and spectrograms will be written. Default: `./output`                                                      |

## Options for verify

**Applies to:** `Batch`, `Verify`

| Config Key      | CLI Flag          | Type                  | Default | Description                                                                                                                                   |
|-----------------|-------------------|-----------------------|---------|-----------------------------------------------------------------------------------------------------------------------------------------------|
| `no_hash_check` | `--no-hash-check` | `bool`                | `false` | Should the hash check of source files be skipped? Note: This is only useful for development and should probably not be used. Default: `false` |
| `exclude_tags`  | `--exclude-tags`  | `Option<Vec<String>>` | -       | Should sources with specific tags be excluded? Default: None                                                                                  |

## Options for transcoding

**Applies to:** `Batch`, `Transcode`, `Upload`, `Verify`

| Config Key          | CLI Flag              | Type                | Default                                                          | Description                                                                                                                                                                                                                                                               |
|---------------------|-----------------------|---------------------|------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `target`            | `--target`            | `Vec<TargetFormat>` | `vec![TargetFormat::Flac, TargetFormat::_320, TargetFormat::V0]` | Formats to attempt to transcode to. Default: `flac`, `320` and `v0`                                                                                                                                                                                                       |
| `allow_existing`    | `--allow-existing`    | `bool`              | `false`                                                          | Allow transcoding to existing formats Note: This is only useful for development and should probably not be used. Default: `false`                                                                                                                                         |
| `sox_random_dither` | `--sox-random-dither` | `bool`              | `false`                                                          | Use random dithering when resampling with `SoX`. By default, `SoX` runs in repeatable mode (`-R`) which seeds the dither random number generator with a fixed value, producing deterministic output. Set this to `true` to use random dithering instead. Default: `false` |

## Options for spectrograms

**Applies to:** `Batch`, `Spectrogram`

| Config Key         | CLI Flag             | Type        | Default                        | Description                                                   |
|--------------------|----------------------|-------------|--------------------------------|---------------------------------------------------------------|
| `spectrogram_size` | `--spectrogram-size` | `Vec<Size>` | `vec![Size::Full, Size::Zoom]` | Sizes of spectrograms to generate. Default: `full` and `zoom` |

## Options for copying files

**Applies to:** `Batch`, `Transcode`, `Upload`

| Config Key  | CLI Flag      | Type   | Default | Description                                                                                                                                                              |
|-------------|---------------|--------|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `hard_link` | `--hard-link` | `bool` | `false` | Should files be hard linked instead of copied? Enabling this option requires the source and destination to be on the same filesystem or mounted volume. Default: `false` |

## Options for image resizing

**Applies to:** `Batch`, `Transcode`

| Config Key             | CLI Flag                 | Type   | Default   | Description                                                                                                                                                                                            |
|------------------------|--------------------------|--------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `no_image_compression` | `--no-image-compression` | `bool` | `false`   | Should compression of images be disabled? Default: `false`                                                                                                                                             |
| `rename_tracks`        | `--rename-tracks`        | `bool` | `false`   | Should transcoded files be renamed from source filenames to a standardized format: `{track:0>N} {title}.{ext}`? Multi-disc releases will be organized into `CD1/`, `CD2/` subfolders. Default: `false` |
| `max_file_size`        | `--max-file-size`        | `u64`  | `750_000` | Maximum file size in bytes beyond which images are compressed. Default: `750000` Only applies to image files.                                                                                          |
| `max_pixel_size`       | `--max-pixel-size`       | `u32`  | `1280`    | Maximum size in pixels for images Default: `1280` Only applied if the image is greater than `max_file_size`.                                                                                           |
| `jpg_quality`          | `--jpg-quality`          | `u8`   | `80`      | Quality percentage to apply for jpg compression. Default: `80` Only applied if the image is greated than `max_file_size`.                                                                              |
| `no_png_to_jpg`        | `--no-png-to-jpg`        | `bool` | `false`   | Should conversion of png images to jpg be disabled? Default: `false` Only applied if the image is greater than `max_file_size`.                                                                        |

## Options for concurrency

**Applies to:** `Batch`, `Spectrogram`, `Transcode`

| Config Key | CLI Flag | Type          | Default | Description                                                         |
|------------|----------|---------------|---------|---------------------------------------------------------------------|
| `cpus`     | `--cpus` | `Option<u16>` | -       | Number of cpus to use for processing. Default: Total number of CPUs |

## Options for upload

**Applies to:** `Batch`, `Upload`

| Config Key                      | CLI Flag                          | Type              | Default | Description                                                                                                                       |
|---------------------------------|-----------------------------------|-------------------|---------|-----------------------------------------------------------------------------------------------------------------------------------|
| `copy_transcode_to_content_dir` | `--copy-transcode-to-content-dir` | `bool`            | `false` | Should the transcoded files be copied to the content directory? Default: `false`                                                  |
| `copy_transcode_to`             | `--copy-transcode-to`             | `Option<PathBuf>` | -       | Directory the transcoded files are copied to. This should be set if you wish to auto-add to your torrent client. Default: Not set |
| `copy_torrent_to`               | `--copy-torrent-to`               | `Option<PathBuf>` | -       | Directory the torrent file is copied to. This should be set if you wish to auto-add to your torrent client. Default: Not set      |
| `dry_run`                       | `--dry-run`                       | `bool`            | `false` | Is this a dry run? If enabled data won't be uploaded and will instead be printed to the console. Default: `false`                 |

## Options for queue cache

**Applies to:** `Batch`, `Queue`

| Config Key | CLI Flag  | Type      | Default                    | Description                                 |
|------------|-----------|-----------|----------------------------|---------------------------------------------|
| `cache`    | `--cache` | `PathBuf` | `PathBuf::from("./cache")` | Path to cache directory. Default: `./cache` |

## Options for batch processing

**Applies to:** `Batch`, `Queue`

| Config Key           | CLI Flag               | Type             | Default | Description                                                                                                                                      |
|----------------------|------------------------|------------------|---------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `spectrogram`        | `--spectrogram`        | `bool`           | `false` | Should the spectrogram command be executed? Default: `false`                                                                                     |
| `transcode`          | `--transcode`          | `bool`           | `false` | Should the transcode command be executed? Default: `false`                                                                                       |
| `retry_transcode`    | `--retry-transcode`    | `bool`           | `false` | Should failed transcodes be retried? Default: `false`                                                                                            |
| `upload`             | `--upload`             | `bool`           | `false` | Should the upload command be executed? Default: `false`                                                                                          |
| `limit`              | `--limit`              | `usize`          | `3`     | Limit the number of torrents to batch process. If `no_limit` is set, this option is ignored. Default: `3`                                        |
| `no_limit`           | `--no-limit`           | `bool`           | `false` | Should the `limit` option be ignored? Default: `false`                                                                                           |
| `wait_before_upload` | `--wait-before-upload` | `Option<String>` | -       | Wait for a duration before uploading the torrent. The duration is a string that can be parsed such as `500ms`, `5m`, `1h30m15s`. Default: `null` |

