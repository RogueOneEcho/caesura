# Command-Line Help for `caesura`

This document contains the help content for the `caesura` command-line program.

**Command Overview:**

* [`caesura`↴](#caesura)
* [`caesura config`↴](#caesura-config)
* [`caesura batch`↴](#caesura-batch)
* [`caesura spectrogram`↴](#caesura-spectrogram)
* [`caesura transcode`↴](#caesura-transcode)
* [`caesura upload`↴](#caesura-upload)
* [`caesura verify`↴](#caesura-verify)

## `caesura`

An all-in-one command line tool to **transcode FLAC** audio files and **upload to gazelle** based indexers/trackers. 

**Usage:** `caesura [COMMAND]`

###### **Subcommands:**

* `config` — Generate a config.json file in the current working directory
* `batch` — Verify, transcode, and upload from multiple FLAC sources in one command
* `spectrogram` — Generate spectrograms for each track of a FLAC source
* `transcode` — Transcode each track of a FLAC source to the target formats
* `upload` — Upload transcodes of a FLAC source
* `verify` — Verify a FLAC source is suitable for transcoding



## `caesura config`

Generate a config.json file in the current working directory

**Usage:** `caesura config`



## `caesura batch`

Verify, transcode, and upload from multiple FLAC sources in one command

**Usage:** `caesura batch [OPTIONS] [SOURCE]`

###### **Arguments:**

* `<SOURCE>` — Source as: torrent id, path to torrent file, or indexer url.

   Examples: `4871992`, `path/to/something.torrent`, `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or `https://example.com/torrents.php?torrentid=4871992`

###### **Options:**

* `--api-key <API_KEY>` — API key with torrent permissions for the indexer
* `--indexer <INDEXER>` — ID of the tracker as it appears in the source field of a torrent.

   Examples: `red`, `pth`, `ops`

   Default: `red`
* `--indexer-url <INDEXER_URL>` — URL of the indexer.

   Examples: `https://redacted.ch`, `https://orpheus.network`

   Default: Dependent on indexer
* `--announce-url <ANNOUNCE_URL>` — Announce URL including passkey

   Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
* `--content <CONTENT>` — Directory containing torrent content.

   Typically this is set as the download directory in your torrent client.

   Default: `./content`
* `--verbosity <VERBOSITY>` — Level of logs to display.

   Default: `info`

  Possible values: `silent`, `error`, `warn`, `info`, `debug`, `trace`

* `--config <CONFIG>` — Path to the configuration file.

   Default: `./config.json`
* `--output <OUTPUT>` — Directory where transcodes and spectrograms will be written.

   Default: `./output`
* `--target <TARGET>` — Formats to attempt to transcode to.

   Default: `flac`, `320` and `v0`

  Possible values: `flac`, `320`, `v0`

* `--allow-existing` — Allow transcoding to existing formats

   Note: This is only useful for development and should probably not be used.

   Default: `false`
* `--no-hash-check` — Should the hash check of source files be skipped?

   Note: This is only useful for development and should probably not be used.

   Default: `false`
* `--cpus <CPUS>` — Number of cpus to use for processing.

   Default: Total number of CPUs
* `--spectrogram-size <SPECTROGRAM_SIZE>` — Sizes of spectrograms to generate.

   Default: `full` and `zoom`

  Possible values: `full`, `zoom`

* `--hard-link` — Should hard links be used when copying files?

   Default: `false`
* `--no-image-compression` — Should compression of images be disabled?

   Default: `false`
* `--max-file-size <MAX_FILE_SIZE>` — Maximum file size in bytes beyond which images are compressed.

   Default: `750000`

   Only applies to image files.
* `--max-pixel-size <MAX_PIXEL_SIZE>` — Maximum size in pixels for images

   Default: `1280`

   Only applied if the image is greater than `max_file_size`.
* `--jpg-quality <JPG_QUALITY>` — Quality percentage to apply for jpg compression.

   Default: `80`

   Only applied if the image is greated than `max_file_size`.
* `--no-png-to-jpg` — Should conversion of png images to jpg be disabled?

   Default: `false`

   Only applied if the image is greater than `max_file_size`.
* `--spectrogram` — Should the spectrogram command be executed?

   Default: `false`
* `--upload` — Should the upload command be executed?

   Default: `false`
* `--limit <LIMIT>` — Limit the number of torrents to batch process.

   If `no_limit` is set, this option is ignored.

   Default: `3`
* `--no-limit` — Should the `limit` option be ignored?

   Default: `false`
* `--wait-before-upload <WAIT_BEFORE_UPLOAD>` — Wait for a duration before uploading the torrent.

   The duration is a string that can be parsed such as `500ms`, `5m`, `1h30m15s`.

   Default: `null`
* `--cache <CACHE>` — Path to cache file.

   Default: `output/cache.json`



## `caesura spectrogram`

Generate spectrograms for each track of a FLAC source

**Usage:** `caesura spectrogram [OPTIONS] [SOURCE]`

###### **Arguments:**

* `<SOURCE>` — Source as: torrent id, path to torrent file, or indexer url.

   Examples: `4871992`, `path/to/something.torrent`, `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or `https://example.com/torrents.php?torrentid=4871992`

###### **Options:**

* `--api-key <API_KEY>` — API key with torrent permissions for the indexer
* `--indexer <INDEXER>` — ID of the tracker as it appears in the source field of a torrent.

   Examples: `red`, `pth`, `ops`

   Default: `red`
* `--indexer-url <INDEXER_URL>` — URL of the indexer.

   Examples: `https://redacted.ch`, `https://orpheus.network`

   Default: Dependent on indexer
* `--announce-url <ANNOUNCE_URL>` — Announce URL including passkey

   Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
* `--content <CONTENT>` — Directory containing torrent content.

   Typically this is set as the download directory in your torrent client.

   Default: `./content`
* `--verbosity <VERBOSITY>` — Level of logs to display.

   Default: `info`

  Possible values: `silent`, `error`, `warn`, `info`, `debug`, `trace`

* `--config <CONFIG>` — Path to the configuration file.

   Default: `./config.json`
* `--output <OUTPUT>` — Directory where transcodes and spectrograms will be written.

   Default: `./output`
* `--spectrogram-size <SPECTROGRAM_SIZE>` — Sizes of spectrograms to generate.

   Default: `full` and `zoom`

  Possible values: `full`, `zoom`

* `--cpus <CPUS>` — Number of cpus to use for processing.

   Default: Total number of CPUs



## `caesura transcode`

Transcode each track of a FLAC source to the target formats

**Usage:** `caesura transcode [OPTIONS] [SOURCE]`

###### **Arguments:**

* `<SOURCE>` — Source as: torrent id, path to torrent file, or indexer url.

   Examples: `4871992`, `path/to/something.torrent`, `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or `https://example.com/torrents.php?torrentid=4871992`

###### **Options:**

* `--api-key <API_KEY>` — API key with torrent permissions for the indexer
* `--indexer <INDEXER>` — ID of the tracker as it appears in the source field of a torrent.

   Examples: `red`, `pth`, `ops`

   Default: `red`
* `--indexer-url <INDEXER_URL>` — URL of the indexer.

   Examples: `https://redacted.ch`, `https://orpheus.network`

   Default: Dependent on indexer
* `--announce-url <ANNOUNCE_URL>` — Announce URL including passkey

   Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
* `--content <CONTENT>` — Directory containing torrent content.

   Typically this is set as the download directory in your torrent client.

   Default: `./content`
* `--verbosity <VERBOSITY>` — Level of logs to display.

   Default: `info`

  Possible values: `silent`, `error`, `warn`, `info`, `debug`, `trace`

* `--config <CONFIG>` — Path to the configuration file.

   Default: `./config.json`
* `--output <OUTPUT>` — Directory where transcodes and spectrograms will be written.

   Default: `./output`
* `--target <TARGET>` — Formats to attempt to transcode to.

   Default: `flac`, `320` and `v0`

  Possible values: `flac`, `320`, `v0`

* `--allow-existing` — Allow transcoding to existing formats

   Note: This is only useful for development and should probably not be used.

   Default: `false`
* `--hard-link` — Should hard links be used when copying files?

   Default: `false`
* `--no-image-compression` — Should compression of images be disabled?

   Default: `false`
* `--max-file-size <MAX_FILE_SIZE>` — Maximum file size in bytes beyond which images are compressed.

   Default: `750000`

   Only applies to image files.
* `--max-pixel-size <MAX_PIXEL_SIZE>` — Maximum size in pixels for images

   Default: `1280`

   Only applied if the image is greater than `max_file_size`.
* `--jpg-quality <JPG_QUALITY>` — Quality percentage to apply for jpg compression.

   Default: `80`

   Only applied if the image is greated than `max_file_size`.
* `--no-png-to-jpg` — Should conversion of png images to jpg be disabled?

   Default: `false`

   Only applied if the image is greater than `max_file_size`.
* `--cpus <CPUS>` — Number of cpus to use for processing.

   Default: Total number of CPUs



## `caesura upload`

Upload transcodes of a FLAC source

**Usage:** `caesura upload [OPTIONS] [SOURCE]`

###### **Arguments:**

* `<SOURCE>` — Source as: torrent id, path to torrent file, or indexer url.

   Examples: `4871992`, `path/to/something.torrent`, `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or `https://example.com/torrents.php?torrentid=4871992`

###### **Options:**

* `--api-key <API_KEY>` — API key with torrent permissions for the indexer
* `--indexer <INDEXER>` — ID of the tracker as it appears in the source field of a torrent.

   Examples: `red`, `pth`, `ops`

   Default: `red`
* `--indexer-url <INDEXER_URL>` — URL of the indexer.

   Examples: `https://redacted.ch`, `https://orpheus.network`

   Default: Dependent on indexer
* `--announce-url <ANNOUNCE_URL>` — Announce URL including passkey

   Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
* `--content <CONTENT>` — Directory containing torrent content.

   Typically this is set as the download directory in your torrent client.

   Default: `./content`
* `--verbosity <VERBOSITY>` — Level of logs to display.

   Default: `info`

  Possible values: `silent`, `error`, `warn`, `info`, `debug`, `trace`

* `--config <CONFIG>` — Path to the configuration file.

   Default: `./config.json`
* `--output <OUTPUT>` — Directory where transcodes and spectrograms will be written.

   Default: `./output`
* `--target <TARGET>` — Formats to attempt to transcode to.

   Default: `flac`, `320` and `v0`

  Possible values: `flac`, `320`, `v0`

* `--allow-existing` — Allow transcoding to existing formats

   Note: This is only useful for development and should probably not be used.

   Default: `false`
* `--copy-transcode-to-content-dir` — Should the transcoded files be copied to the content directory?

   This should be enabled if you wish to auto-add to your torrent client.

   Default: `false`
* `--copy-torrent-to <COPY_TORRENT_TO>` — Directory the torrent file is copied to.

   This should be set if you wish to auto-add to your torrent client.

   Default: Not set
* `--hard-link` — Should files be hard linked instead of copied?

   Enabling this option requires the source and destination to be on the same filesystem or mounted volume.

   Default: `false`
* `--dry-run` — Is this a dry run?

   If enabled data won't be uploaded and will instead be printed to the console.

   Default: `false`



## `caesura verify`

Verify a FLAC source is suitable for transcoding

**Usage:** `caesura verify [OPTIONS] [SOURCE]`

###### **Arguments:**

* `<SOURCE>` — Source as: torrent id, path to torrent file, or indexer url.

   Examples: `4871992`, `path/to/something.torrent`, `https://example.com/torrents.php?id=2259978&torrentid=4871992#torrent4871992`, or `https://example.com/torrents.php?torrentid=4871992`

###### **Options:**

* `--api-key <API_KEY>` — API key with torrent permissions for the indexer
* `--indexer <INDEXER>` — ID of the tracker as it appears in the source field of a torrent.

   Examples: `red`, `pth`, `ops`

   Default: `red`
* `--indexer-url <INDEXER_URL>` — URL of the indexer.

   Examples: `https://redacted.ch`, `https://orpheus.network`

   Default: Dependent on indexer
* `--announce-url <ANNOUNCE_URL>` — Announce URL including passkey

   Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
* `--content <CONTENT>` — Directory containing torrent content.

   Typically this is set as the download directory in your torrent client.

   Default: `./content`
* `--verbosity <VERBOSITY>` — Level of logs to display.

   Default: `info`

  Possible values: `silent`, `error`, `warn`, `info`, `debug`, `trace`

* `--config <CONFIG>` — Path to the configuration file.

   Default: `./config.json`
* `--output <OUTPUT>` — Directory where transcodes and spectrograms will be written.

   Default: `./output`
* `--target <TARGET>` — Formats to attempt to transcode to.

   Default: `flac`, `320` and `v0`

  Possible values: `flac`, `320`, `v0`

* `--allow-existing` — Allow transcoding to existing formats

   Note: This is only useful for development and should probably not be used.

   Default: `false`
* `--no-hash-check` — Should the hash check of source files be skipped?

   Note: This is only useful for development and should probably not be used.

   Default: `false`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
