# <p style="text-align: center">caesura 𝄓</p>

A versatile command line tool for fully automated transcoding of FLAC sources.

*`caesura` will crunch through your torrent directory and automatically transcode any suitable FLAC sources.*

## Features

- RED and OPS fully supported.
- Runs on Linux and macOS. Windows via WSL or Docker.
- Fully configurable, if there's something hard coded that you think should be configurable then [open a discussion on GitHub](https://github.com/RogueOneEcho/caesura/discussions).
- Install via [Docker](docs/DOCKER.md), [Homebrew](docs/INSTALL.md#homebrew-linux-and-macos), [Nix](docs/INSTALL.md#nix-linux-and-macos) and [standalone binaries](docs/INSTALL.md#github-releases-linux-and-macos).
- Minimal CLI tools required. Only FLAC, LAME and SoX. Image resizing, torrent creation and audio metadata inspection are handled natively.

### Source Verification

Each source is verified to ensure:
- A lossless FLAC
- Not a scene, lossy, unconfirmed or trumpable release
- Files match the torrent hash
- Audio tags for artist, album, title and track number are set
- Classical sources have a composer tag
- Vinyl track numbering is converted to numeric
- Sample rate and channels are suitable

### Inspect

- Display audio properties, tags and embedded pictures for every track in a directory
- Supports FLAC and MP3

### Spectrogram Generation

- Full and zoomed spectrograms generated for review

### Transcoding

- Multi-threaded transcoding with optional CPU limit
- FLAC and FLAC 24 bit sources are supported
- FLAC, MP3 320 (CBR) and MP3 V0 (VBR) target formats
- Existing formats are skipped
- Nested sub directories are fully supported (i.e. CD1 and CD2 etc)
- Automatic naming following established conventions, with decoding of HTML entities
- Automatic torrent file creation
- Images in the root and first nested directory are included and all other files ignored
- Images larger than 750 KB are reduced to less than 1280 px, converted to JPG and compressed
- Optional track renaming to standardized filenames
- Optional stripping of Vorbis comments from transcoded output

### Upload

- Copy transcodes to content directory
- Torrent injection via client auto-add directory

### Publish

- Source FLAC publish workflow for RED via manifest
- Optional source seeding prep (hard-link by default, `--move-source` to move)
- Auto-generated BBCode release description with notes, details and hidden tags
- Optional source torrent injection via client auto-add directory

### Batch / Queue

- Verify, transcode and upload with one command for every torrent file in a directory
- Source torrents are added to a queue to track their progress, reducing duplicate work and speeding up subsequent runs

## Documentation

| Guide                                      | Description                                          |
|--------------------------------------------|------------------------------------------------------|
| **[DOCKER.md](docs/DOCKER.md)**                 | Docker installation (recommended)                    |
| **[INSTALL.md](docs/INSTALL.md)**               | Native installation                                  |
| **[DEPENDENCIES.md](docs/DEPENDENCIES.md)**     | External tool dependencies (FLAC, LAME, SoX_ng)     |
| **[SETUP.md](docs/SETUP.md)**                   | Configuration, default paths, multi-indexer          |
| **[COMMANDS.md](docs/COMMANDS.md)**              | Command guide with examples for every subcommand     |
| **[OPTIONS.md](docs/OPTIONS.md)**                | CLI flags and `config.yml` file options              |
| **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** | Troubleshooting guide                              |
| **[COMPARISON.md](docs/COMPARISON.md)**          | Comparison with similar tools                        |
| **[DIRECTORIES.md](docs/DIRECTORIES.md)**       | Explanation of writable directories                  |
| **[BUILD.md](docs/BUILD.md)**                        | Building from source                                 |
| **[TESTING.md](docs/TESTING.md)**               | Test infrastructure and conventions                  |
| **[CONTRIBUTING.md](CONTRIBUTING.md)**           | Contributing guidelines                              |

## Releases and Changes

Releases and a full changelog are available via [GitHub Releases](https://github.com/RogueOneEcho/caesura/releases).

## History

[**DevYukine**](https://github.com/DevYukine) completed the **initial work** and released it as [**red_oxide**](https://github.com/DevYukine/red_oxide) under an [MIT license](https://github.com/DevYukine/red_oxide/blob/master/LICENSE).

[**RogueOneEcho**](https://github.com/RogueOneEcho) forked the project to make small iterative improvements that evolved into a complete rewrite. The fork is released as [**caesura**](https://github.com/RogueOneEcho/caesura) under an [AGPL license](LICENSE.md).

See also the list of
[contributors](https://github.com/RogueOneEcho/caesura/contributors)
who participated in this project.

---

**Next: [Installation](docs/INSTALL.md)**
