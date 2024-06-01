## About

An all-in-one command line tool to **transcode FLAC** audio files and **upload to gazelle** based indexers/trackers. 

## Features

All gazelle based indexers/trackers are supported
- RED
- **[[new](https://github.com/DevYukine/red_oxide/issues/7)]** OPS.

Tested on Linux, theoretically works on Windows.

Fully configurable, if there's something hard coded that you think should be configurable then open issue on GitHub.

### Source Verification

Each source is verified to ensure it's:
- A lossless FLAC
- Not a scene or lossy release
- Files match the torrent hash
- Audio tags for artist, album, title and track number are set
- **[[fixed](https://github.com/DevYukine/red_oxide/issues/18)]** Vinyl track numbering
- Sample rate and channels are supported

### Spectrogram Generation

- Full and zoomed spectrograms generated for review

### Transcoding

- **[fixed]** Multi-threaded transcoding with optional CPU limit
- FLAC and FLAC 24 bit sources are supported
- FLAC, MP3 320 (CBR) and MP3 V0 (VBR) target formats
- Existing formats are skipped
- **[[fixed](https://github.com/DevYukine/red_oxide/issues/21)]** Nested sub directories are fully supported (i.e. CD1, and CD2 etc)
- **[[fixed](https://github.com/DevYukine/red_oxide/issues/22)]** Automatic naming following established conventions, with decoding of HTML entities.
- **[[fixed](https://github.com/DevYukine/red_oxide/issues/24)]** Shorter file names.
- Automatic torrent file creation
- **[new]** Images in the root directory are included and all other files ignored.
- **[new]** Images larger than 750 KB are (optionally) compressed and reduced to less than 1920 px. 

*The logic being that folder and cover images are included but to minimize file size, but for artwork and anything additional the original source can be downloaded*

### Upload

*Work in progress*

## Getting started

### Install

#### Linux

1. [Install Rust](https://www.rust-lang.org/tools/install)

2. [Install Intermodal](https://github.com/casey/intermodal#installation)

```bash
cargo install imdl
```

3. Install FLAC, LAME, SOX and ImageMagick

```bash
sudo apt install flac lame sox imagemagick --yes
```

4. Install red_oxide
```bash
cargo install red_oxide
```

#### Windows

*To be confirmed*

#### Docker

*To be confirmed*

### CLI Commands

#### Verify source `rogue_oxide verify`

#### Generate spectrograms `rogue_oxide spectrogram`

#### Transcode FLACs `rogue_oxide transcode`

You have to specify api-key, torrent-directory, content-directory, transcode-directory & spectrogram-directory either via the config file or via the CLI

```
Transcode FLACs to other co-existing formats

Usage: red_oxide transcode [OPTIONS] [URLS]...

Arguments:
  [URLS]...  The Perma URLs (PL's) of torrents to transcode

Options:
      --debug
          If debug logs should be shown
  -a, --automatic-upload
          If the upload should be done automatically
      --concurrency <CONCURRENCY>
          How many tasks (for transcoding as example) should be run in parallel, defaults to your CPU count
      --api-key <API_KEY>
          The Api key from Redacted to use there API with
      --content-directory <CONTENT_DIRECTORY>
          The path to the directory where the downloaded torrents are stored
      --transcode-directory <TRANSCODE_DIRECTORY>
          The path to the directory where the transcoded torrents should be stored
      --torrent-directory <TORRENT_DIRECTORY>
          The path to the directory where the torrents should be stored
      --spectrogram-directory <SPECTROGRAM_DIRECTORY>
          The path to the directory where the spectrograms should be stored
  -c, --config-file <CONFIG_FILE>
          The path to the config file
  -f, --allowed-transcode-formats <ALLOWED_TRANSCODE_FORMATS>
          List of allowed formats to transcode to, defaults to all formats if omitted [possible values: flac24, flac, mp3320, mp3-v0]
  -m, --move-transcode-to-content
          If the transcode should be moved to the content directory, useful when you want to start seeding right after you upload
      --skip-hash-check
          If the hash check of the original torrent should be skipped, defaults to false, not recommended and if enabled done at own risk!
      --skip-spectrogram
          If the spectrogram check of the original torrent should be skipped, defaults to false, not recommended and if enabled done at own risk!
  -d, --dry-run
          If this is a dry run, no files will be uploaded to Redacted
  -h, --help
          Print help

```

### Config file

This is useful if you don't want a super long CLI command and your configs do not change often, note that all the options can be specified via the CLI as well and are fully optional in this config file (will be merged with the CLI options if specified)

There are multiple default locations where the config file will be searched for, in this order (once found it will not look for the config file in the other locations):
1. The path specified via the --config-file CLI option
2. `./red_oxide.config.json` (In the same folder as the red_oxide executable)
3. `%APPDATA%/red_oxide/red_oxide.config.json` (only on Windows)
4. `$XDG_CONFIG_HOME/red_oxide/red_oxide.config.json`
5. `HOME/.config/red_oxide/red_oxide.config.json`
6. `HOME/red_oxide.config.json`

HOME is determined by these environment variables on Windows in this order:
1. `%HOME%`
2. `%USERPROFILE%`
3. `%HOMEDRIVE%\%HOMEPATH%`

HOME is determined by these environment variables on Linux in this order:
1. `$HOME`


```json
{
  "api_key": "YOUR_API_KEY",
  "torrent_directory": "FULL_PATH_WHERE_TORRENT_FILES_WILL_BE_STORED",
  "content_directory": "FULL_PATH_WHERE_CONTENT_IS_LOCATED",
  "transcode_directory": "FULL_PATH_WHERE_TRANSCODED_CONTENT_WILL_BE_PUT",
  "spectrogram_directory": "FULL_PATH_WHERE_SPECTROGRAMS_WILL_BE_PUT",
  "move_transcode_to_content": true,
  "automatic_upload": true,
  "skip_hash_check": false,
  "skip_spectrogram": false,
  "allowed_transcode_formats": ["Flac", "Mp3320", "Mp3V0"],
  "concurrency": 16
}

```

## Releases and Changes

All release versions follow the [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) specification.

All commit messages follow the [Conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) specification.

Releases and a full changelog are available via [GitHub Releases](https://github.com/RogueOneEcho/rogue_oxide/releases).

## History

[**DevYukine**](https://github.com/DevYukine) completed the **initial work** and released it as [**red_oxide**](https://github.com/DevYukine/red_oxide) under an [MIT license](LICENSE.HISTORIC.md).

[**RogueOneEcho**](https://github.com/RogueOneEcho) then forked the project to complete a major refactor, **fix some issues**, add **new features** and improve logging and error handling. The fork is released as [**rogue_oxide**](https://github.com/RogueOneEcho/rogue_oxide) under an [AGPL license](LICENSE.md).

*The main difference between the former MIT license and the present AGPL license is that if you intend to distribute a modified version of the code - even to run it on a server - you must also provide the modified source code under an AGPL license.*

*This is often known as copyleft. The intent is to ensure that anyone taking advantage of this open source work are also contributing back to the open source community.*

The code base has now adopted [object oriented patterns](https://refactoring.guru/design-patterns/catalog) with SOLID principles.

See also the list of
[contributors](https://github.com/DevYukine/red_oxide/contributors)
who participated in this project.
