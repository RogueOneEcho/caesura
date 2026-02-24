# Options Reference

This document describes all configuration options available in caesura.
Options can be set via CLI flags or in `config.yml`.

## Configuration file path.

Commands:
- `batch`
- `config`
- `queue add`
- `queue list`
- `queue rm`
- `queue summary`
- `spectrogram`
- `transcode`
- `upload`
- `verify`

| YAML Key | CLI Flag   | Type              | Default                                               | Description                     |
| -------- | ---------- | ----------------- | ----------------------------------------------------- | ------------------------------- |
| `config` | `--config` | `Option<PathBuf>` | `~/.config/caesura/config.yml` or platform equivalent | Path to the configuration file. |

## Options shared by all commands

Commands:
- `batch`
- `queue add`
- `queue list`
- `queue rm`
- `queue summary`
- `spectrogram`
- `transcode`
- `upload`
- `verify`

| YAML Key       | CLI Flag         | Type           | Default                                                 | Description                                                                                                                             |
| -------------- | ---------------- | -------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `announce_url` | `--announce-url` | `String`       | `""`                                                    | Announce URL including passkey<br>Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce` |
| `api_key`      | `--api-key`      | `String`       | `""`                                                    | API key with torrent permissions for the indexer.                                                                                       |
| `indexer`      | `--indexer`      | `String`       | from announce_url                                       | ID of the tracker as it appears in the source field of a torrent.<br>Examples: `red`, `pth`, `ops`                                      |
| `indexer_url`  | `--indexer-url`  | `String`       | from announce_url                                       | URL of the indexer.<br>Examples: `https://redacted.sh`, `https://orpheus.network`                                                       |
| `content`      | `--content`      | `Vec<PathBuf>` | `[]`                                                    | Directories containing torrent content.<br>Typically this is set as the download directory in your torrent client.                      |
| `verbosity`    | `--verbosity`    | `Verbosity`    | `"info"`                                                | Level of logs to display.                                                                                                               |
| `log_time`     | `--log-time`     | `TimeFormat`   | `"local"`                                               | Time format to use in logs.                                                                                                             |
| `output`       | `--output`       | `PathBuf`      | `~/.local/share/caesura/output/` or platform equivalent | Directory where transcodes and spectrograms will be written.                                                                            |

## Options for verify

Commands:
- `batch`
- `verify`

| YAML Key        | CLI Flag          | Type                  | Default | Description                                                                                                                     |
| --------------- | ----------------- | --------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `no_hash_check` | `--no-hash-check` | `bool`                | `false` | Should the hash check of source files be skipped?<br>Note: This is only useful for development and should probably not be used. |
| `exclude_tags`  | `--exclude-tags`  | `Option<Vec<String>>` | ~       | Should sources with specific tags be excluded?                                                                                  |

## Options for transcoding

Commands:
- `batch`
- `transcode`
- `upload`
- `verify`

| YAML Key                  | CLI Flag                    | Type                | Default                                 | Description                                                                                                                                                                                                                                                       |
| ------------------------- | --------------------------- | ------------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `target`                  | `--target`                  | `Vec<TargetFormat>` | `["flac","320","v0"]`                   | Formats to attempt to transcode to.                                                                                                                                                                                                                               |
| `allow_existing`          | `--allow-existing`          | `bool`              | `false`                                 | Allow transcoding to existing formats.<br>Note: This is only useful for development and should probably not be used.                                                                                                                                              |
| `sox_random_dither`       | `--sox-random-dither`       | `bool`              | `false`                                 | Use random dithering when resampling with `SoX`.<br>By default, `SoX` runs in repeatable mode (`-R`) which seeds the dither<br>random number generator with a fixed value, producing deterministic output.<br>Set this to `true` to use random dithering instead. |
| `exclude_vorbis_comments` | `--exclude-vorbis-comments` | `Vec<String>`       | `["COMMENT","ENCODER","RATING","WORK"]` | Vorbis comment tag names to exclude from transcoded output.                                                                                                                                                                                                       |

## Options for spectrograms

Commands:
- `batch`
- `spectrogram`

| YAML Key           | CLI Flag             | Type        | Default           | Description                        |
| ------------------ | -------------------- | ----------- | ----------------- | ---------------------------------- |
| `spectrogram_size` | `--spectrogram-size` | `Vec<Size>` | `["full","zoom"]` | Sizes of spectrograms to generate. |

## Options for sox binary selection

Commands:
- `batch`
- `spectrogram`
- `transcode`
- `version`

| YAML Key      | CLI Flag        | Type         | Default       | Description                                        |
| ------------- | --------------- | ------------ | ------------- | -------------------------------------------------- |
| `sox_variant` | `--sox-variant` | `SoxVariant` | auto-detected | `SoX` binary to use.<br>Options: `sox` or `sox_ng` |

## Options for copying files

Commands:
- `batch`
- `transcode`
- `upload`

| YAML Key    | CLI Flag      | Type   | Default | Description                                                                                                                                                |
| ----------- | ------------- | ------ | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hard_link` | `--hard-link` | `bool` | `false` | Should files be hard linked instead of copied?<br>Enabling this option requires the source and destination to be on the same filesystem or mounted volume. |

## Options for image resizing

Commands:
- `batch`
- `transcode`

| YAML Key               | CLI Flag                 | Type   | Default  | Description                                                                                                                                                                                                                                                                      |
| ---------------------- | ------------------------ | ------ | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `no_image_compression` | `--no-image-compression` | `bool` | `false`  | Should compression of images be disabled?                                                                                                                                                                                                                                        |
| `rename_tracks`        | `--rename-tracks`        | `bool` | `false`  | Should transcoded files be renamed?<br>If enabled then tracks are renamed into a standardized format: `{number} {title}.{ext}`.<br>Multi-disc releases will be organized into `CD1/`, `CD2/` subfolders.<br>- `1 Example track title.flac`<br>- `CD1/10 Example track title.mp3` |
| `max_file_size`        | `--max-file-size`        | `u64`  | `750000` | Maximum file size in bytes beyond which images are compressed.<br>Only applies to image files.                                                                                                                                                                                   |
| `max_pixel_size`       | `--max-pixel-size`       | `u32`  | `1280`   | Maximum size in pixels for images.<br>Only applied if the image is greater than `max_file_size`.                                                                                                                                                                                 |
| `jpg_quality`          | `--jpg-quality`          | `u8`   | `80`     | Quality percentage to apply for jpg compression.<br>Only applied if the image is greater than `max_file_size`.                                                                                                                                                                   |
| `no_png_to_jpg`        | `--no-png-to-jpg`        | `bool` | `false`  | Should conversion of png images to jpg be disabled?<br>Only applied if the image is greater than `max_file_size`.                                                                                                                                                                |

## Options for concurrency

Commands:
- `batch`
- `spectrogram`
- `transcode`

| YAML Key | CLI Flag | Type          | Default    | Description                           |
| -------- | -------- | ------------- | ---------- | ------------------------------------- |
| `cpus`   | `--cpus` | `Option<u16>` | Total CPUs | Number of cpus to use for processing. |

## Options for upload

Commands:
- `batch`
- `upload`

| YAML Key                        | CLI Flag                          | Type              | Default | Description                                                                                                         |
| ------------------------------- | --------------------------------- | ----------------- | ------- | ------------------------------------------------------------------------------------------------------------------- |
| `copy_transcode_to_content_dir` | `--copy-transcode-to-content-dir` | `bool`            | `false` | Should the transcoded files be copied to the content directory?                                                     |
| `copy_transcode_to`             | `--copy-transcode-to`             | `Option<PathBuf>` | ~       | Directory the transcoded files are copied to.<br>This should be set if you wish to auto-add to your torrent client. |
| `copy_torrent_to`               | `--copy-torrent-to`               | `Option<PathBuf>` | ~       | Directory the torrent file is copied to.<br>This should be set if you wish to auto-add to your torrent client.      |
| `dry_run`                       | `--dry-run`                       | `bool`            | `false` | Is this a dry run?<br>If enabled data won't be uploaded and will instead be printed to the console.                 |

## Options for queue cache

Commands:
- `batch`
- `queue add`
- `queue list`
- `queue rm`
- `queue summary`

| YAML Key | CLI Flag  | Type      | Default                                    | Description              |
| -------- | --------- | --------- | ------------------------------------------ | ------------------------ |
| `cache`  | `--cache` | `PathBuf` | `~/.cache/caesura/` or platform equivalent | Path to cache directory. |

## Options for batch processing

Commands:
- `batch`
- `queue list`

| YAML Key             | CLI Flag               | Type             | Default | Description                                                                                                                         |
| -------------------- | ---------------------- | ---------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `spectrogram`        | `--spectrogram`        | `bool`           | `false` | Should the spectrogram command be executed?                                                                                         |
| `transcode`          | `--transcode`          | `bool`           | `false` | Should the transcode command be executed?                                                                                           |
| `retry_transcode`    | `--retry-transcode`    | `bool`           | `false` | Should failed transcodes be retried?                                                                                                |
| `upload`             | `--upload`             | `bool`           | `false` | Should the upload command be executed?                                                                                              |
| `limit`              | `--limit`              | `usize`          | `3`     | Limit the number of torrents to batch process.<br>If `no_limit` is set, this option is ignored.                                     |
| `no_limit`           | `--no-limit`           | `bool`           | `false` | Should the `limit` option be ignored?                                                                                               |
| `wait_before_upload` | `--wait-before-upload` | `Option<String>` | ~       | Wait for a duration before uploading the torrent.<br>The duration is a string that can be parsed such as `500ms`, `5m`, `1h30m15s`. |

## Options for `queue add` command.

Commands:
- `queue add`

| YAML Key         | CLI Flag | Type              | Default | Description                                                                                                                                                                                                                                                                                                                                                                                                                |
| ---------------- | -------- | ----------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `queue_add_path` | ~        | `Option<PathBuf>` | ~       | A path to either a directory of `.torrent` files or a single YAML queue file.<br>If you set this to the directory your torrent client stores `.torrent` files then caesura<br>will automatically load everything from your client.<br>- For qBittorrent use the `BT_backup` directory<br>- For deluge use the `state` directory<br>Examples:<br>- `/srv/qBittorrent/BT_backup`<br>- `/srv/deluge/state`<br>- `./queue.yml` |
