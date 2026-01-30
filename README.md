# <p style="text-align: center">caesura 𝄓</p>

A versatile command line tool for automated verifying and transcoding of all your torrents.

## Features

Most gazelle based indexers/trackers are supported
- RED
- **[[new](https://github.com/RogueOneEcho/caesura/issues/7)]** OPS.

Tested on Linux, theoretically works on Windows.

Fully configurable, if there's something hard coded that you think should be configurable then [open a discussion on GitHub](https://github.com/RogueOneEcho/caesura/discussions).

### Source Verification

Each source is verified to ensure:
- A lossless FLAC
- Not a scene, lossy, unconfirmed, or trumpable release
- Files match the torrent hash
- Audio tags for artist, album, title and track number are set
- **[[fixed](https://github.com/RogueOneEcho/caesura/issues/47)]** Classical sources have a composer tag.
- **[[fixed](https://github.com/RogueOneEcho/caesura/issues/18)]** Vinyl track numbering is converted to numeric
- Sample rate and channels are suitable

### Spectrogram Generation

- Full and zoomed spectrograms generated for review

### Transcoding

- **[fixed]** Multi-threaded transcoding with optional CPU limit
- FLAC and FLAC 24 bit sources are supported
- FLAC, MP3 320 (CBR) and MP3 V0 (VBR) target formats
- Existing formats are skipped
- **[[fixed](https://github.com/RogueOneEcho/caesura/issues/21)]** Nested sub directories are fully supported (i.e. CD1, and CD2 etc)
- **[[fixed](https://github.com/RogueOneEcho/caesura/issues/22)]** Automatic naming following established conventions, with decoding of HTML entities.
- **[[fixed](https://github.com/RogueOneEcho/caesura/issues/24)]** Shorter file names.
- Automatic torrent file creation
- **[new]** Images in the root and first nested directory are included and all other files ignored.
- **[new]** Images larger than 750 KB are reduced to less than 1280 px, converted to JPG and compressed.

### Upload

- Copy transcodes to content directory
- Copy torrent file to client auto-add directory

### Batch / Queue

- **[new]** Verify, transcode and upload with one command for every torrent file in a directory.
- **[new]** Source torrents are added to a queue to track their progress reducing duplicate work and speeding up subsequent runs.

*The application will crunch through your torrent directory and automatically determine which are FLAC sources suitable for transcoding.*

## Getting started

Docker is the recommended way to run the application across all platforms.
- All dependencies are built into the image
- Runs in an isolated environment reducing risks to your system

> [!TIP]
> **[Configuration options and the commands they apply to are documented in CONFIG.md](CONFIG.md)**

### 0. Install Docker

[Install Docker Engine](https://docs.docker.com/engine/install/) for your OS.

### 1. Run the `help` command

Run the `help` command to see the available commands and options.

```bash
docker run ghcr.io/rogueoneecho/caesura --help
```

> [!TIP]
> You can append `--help` to any command to see the available options.
>
> ```bash
> docker run ghcr.io/rogueoneecho/caesura verify --help
> ```

### 2. Create a configuration file

Create a `config.yml` file with the following content:

- `announce_url` Your personal announce URL. Find it on upload page.
- `api_key` Create an API key with `Torrents` permission `Settings > Access Settings > Create an API Key`

Refer to [CONFIG.md](CONFIG.md) for full documentation of options.

```yaml
announce_url: https://flacsfor.me/YOUR_ANNOUNCE_KEY/announce
api_key: "YOUR_API_KEY"
```

You can then run the `config` command to see how the full configuration including default values the application will use:

> [!NOTE]
> Because the application is running in a Docker container, you need to mount the config file as a volume.

```bash
docker run -v ./config.yml:/config.yml ghcr.io/rogueoneecho/caesura config
```

> [!TIP]
> The following fields are optional, if not set they're set based on the `announce_url`:
> - `indexer` the id of the indexer: `red`, `pth`, `ops`.
> - `indexer_url` the URL of the indexer: `https://redacted.sh`, `https://orpheus.network`.

### 3. Create storage directories

Create a directory for the application to output files to:

```bash
mkdir ./output
```

Create a directory for the application to cache files to:

```bash
mkdir ./cache
```

> [!TIP]
> Refer to the [directory structure](#directory-structure) section for documentation on the purpose and structure of these directories.

### 4. Verify a source

Run the `verify` command with the source as an argument.

> [!NOTE]
> Because the application is running in a Docker container, you need to mount the config file, content directory, output directory and cache directory.

> [!TIP]
> For the source you can use a permalink, the numeric torrent id or a path to a torrent file:
>
> Each step of this guide will use a different source to demonstrate, but feel free to use whichever suits you best.

```bash
docker run \
-v ./config.yml:/config.yml \
-v /path/to/your/content:/content \
-v ./output:/output \
-v ./cache:/cache \
ghcr.io/rogueoneecho/caesura \
verify https://redacted.sh/torrents.php?id=80518&torrentid=142659#torrent142659
```

If it looks good you can proceed to the next step, otherwise try another source.

### 5. Use Docker Compose

Docker is great but specifying the volumes everytime is tedious and prone to error.

Using Docker Compose simplifies this by storing the configuration in a `docker-compose.yml` file.

Create a `docker-compose.yml` file with the following content:

```yaml
services:
  caesura:
    container_name: caesura
    image: ghcr.io/rogueoneecho/caesura
    volumes:
    - ./config.yml:/config.yml:ro
    - /path/to/your/content:/content:ro
    - ./output:/output
    - ./cache:/cache
```

> [!NOTE]
> The `:ro` suffix makes the volume read-only which is a good security practice.
>
> If you intend to use the `--copy-transcode-to-content-dir` option then you must remove the `:ro` suffix from the content volume.
>
> If you intend to use the `--hard-link` option then the `content` and `output` paths must be inside the same volume and you will need to update the `config.yml` accordingly.

Now run the verify command again but this time using Docker Compose:

```bash
docker compose run --rm caesura verify 142659
```

### 6. Generate spectrograms of a source

Run the `spectrogram` command with the source as an argument.

```bash
docker compose run --rm caesura spectrogram 142659
```

Inspect the spectrograms in the output directory.

### 7. Transcode a source

Run the `transcode` command with the source as an argument.

```bash
docker compose run --rm caesura transcode "Khotin - Hello World [2014].torrent"
```

Inspect the transcodes in the output directory.

> [!TIP]
> Things to check:
> - Folder structure
> - File names
> - Tags
> - Audio quality
> - Image size and compression quality

Two `.torrent` files are created for each transcode, one with the indexer as a suffix (`*.red.torrent` or `*.ops.torrent`) and one without. For now these files are identical but if you subsequently transcode the same source for a different indexer they may differ.

### 8. Upload transcodes

> [!WARNING]
> You are responsible for everything you upload.
>
> Misuse of this application can result in the loss of your upload privileges.

Run the `upload` command with the source as an argument.

> [!TIP]
> Ideally you've already checked everything and nothing will go wrong but just in case there is a grace period after uploading in which you can remove the upload from your indexer.

> [!TIP]
> If you're unsure about this then you can append `--dry-run` to the command and instead of uploading it will print the data that would be submitted.

```bash
docker compose run --rm caesura upload https://redacted.sh/torrents.php?id=80518&torrentid=142659#torrent142659
```

If you haven't already then add the `*.red.torrent` or `*.ops.torrent` file to your torrent client.

> [!TIP]
> `caesura` can automatically add the `.torrent` to your torrent client if it supports an autoadd directory.
>
> Either use the `--copy-torrent-to path/to/autoadd/directory` CLI option or add the following to `config.yml`
>
> ```yaml
> copy_torrent_to: path/to/autoadd/directory
> ```
>
> Don't forget to ensure the path is mounted as a volume in `docker-compose.yml`.
>
> In qBittorrent you can configure auto add under: Options > Downloads > Automatically add torrents from
>
> Monitored Folder: `path/to/autoadd/directory`
>
> Override save location: `path/to/caesura/output`

Go to your indexer and check your uploads to make sure everything has gone to plan.

### 9. Batch processing

> [!WARNING]
> You are responsible for everything you upload.
>
> Misuse of this application, especially the `batch` command, can result in the loss of your upload privileges or a ban.

Now that you have the hang of the application we can speed things up with the `queue` and `batch` commands.

The `batch` command handles `verify`, `spectrogram`, `transcode` and `upload` in a single command.

Run the `queue add` command to search through a directory of torrents and queue them for batch processing:

> [!NOTE]
> The `batch` and `queue` commands use the cache directory to store progress helping speed up subsequent runs.
>
> Make sure the cache directory is in a mounted volume so it's not deleted between runs.

```bash
docker compose run --rm caesura queue add /path/to/your/torrents
```

Run the `queue list` command to see what is next in the queue for the current `indexer`:

```bash
docker compose run --rm caesura queue list
```

By default the `batch` command will limit to processing just `3` transcodes and it won't create spectrograms or upload unless explicitly instructed. These safeguards are in place to prevent mistakenly uploading a bunch of sources that you haven't checked.

Run the command to batch verify and transcode the three sources in the queue:

```bash
docker compose run --rm caesura batch --transcode
```

> [!TIP]
> Add the `--spectrogram` flag to generate spectrograms.

If everything goes to plan three sources should have transcoded to your output directory.

Use the `queue summary` command to see the progress:

```bash
docker compose run --rm caesura queue summary
```

> [!TIP]
> Refer to the [analyzing the queue](#analyzing-the-queue) section to inspect the queue in greater detail.

Nothing was uploaded in the previous run of the `batch` command giving you a chance to check the transcodes and spectrograms. Once you're satisfied run again but with the `--upload` flag.

```bash
docker compose run --rm caesura batch --transcode --upload
```

Check the uploads on your indexer to make sure everything has gone to plan.

Now, we can set the batch command loose with the `--no-limit` option to transcode (but not upload) every source in the directory:

```bash
docker compose run --rm caesura batch --transcode --no-limit
```

Once you've checked the transcodes you can start to upload them in batches. The `--wait-before-upload 30s` option will add a 30 second wait interval between uploads to give you time to check everything looks good, and spread out the load on your indexer:

```bash
docker compose run --rm caesura batch --upload --limit 10 --wait-before-upload 30s
```

> [!WARNING]
> In theory you can execute with both `--upload --no-limit` but that is probably a bad idea and a very fast way to lose your upload privileges.
>
> If you are going to do so then you should definitely use a long wait interval:
> `--upload --no-limit --wait-before-upload 2m`

### 10. Next steps

Check out the [full documentation of configuration options in CONFIG.md](CONFIG.md), in particular you may want to use `--copy-transcode-to-content-dir` and `--copy-torrent-to` to suit your preferred setup.

### 11. Upload to both RED and OPS

`caesura` is designed to work with both `RED` and `OPS`. There's no need for separate cache or output directories, however, you will need a separate configuration for each and the commands must be run separately.

Make a copy of `config.yml` for each indexer. For clarity I recommend naming them `config.red.yml` and `config.ops.yml`

```bash
cp config.yml config.red.yml
cp config.yml config.ops.yml
```

Edit each config file to include the API key and announce URL for that indexer.

```yaml
announce_url: https://home.opsfet.ch/YOUR_ANNOUNCE_KEY/announce
api_key: "YOUR_API_KEY"
```

Edit `docker-compose.yml` to include separate services for each indexer. The only difference between them is the mapping of the the config file.

```yaml
services:

  caesura-red:
    container_name: caesura-red
    image: ghcr.io/rogueoneecho/caesura
    volumes:
    - ./config.red.yml:/config.yml:ro
    - /path/to/your/content:/content:ro
    - ./output:/output
    - ./cache:/cache

  caesura-ops:
    container_name: caesura-ops
    image: ghcr.io/rogueoneecho/caesura
    volumes:
    - ./config.ops.yml:/config.yml:ro
    - /path/to/your/content:/content:ro
    - ./output:/output
    - ./cache:/cache
```

Run the config command to verify the config is loaded correctly for each:

```bash
docker compose run --rm caesura-red config
docker compose run --rm caesura-ops config
```

The `queue` command is indexer agnostic so as long as both configurations use the same `cache` it only needs to be run once.

```bash
docker compose run --rm caesura-red queue add /path/to/your/torrents
```

Then run the `batch` command for each indexer:

```bash
docker compose run --rm caesura-red batch --transcode --upload
docker compose run --rm caesura-ops batch --transcode --upload
```

> [!NOTE]
> If you start a transcode for a source on OPS that you've already transcoded for RED then `caesura` will detect this automatically and instead of re-transcoding it simply creates a `*.ops.torrent` file from the existing transcode so there's no duplication of effort and the existing files are re-used without taking up additional space.
>
> Therefore the first time you run the `batch` command for the new indexer you will likely see a few messages along the lines of:
>
> ```
> Found existing 320 transcode
> Found existing V0 transcode
> ```

## Directory Structure

The application requires two writable directories.

### Cache Directory

The `verify` command will download `.torrent` files for each source to `{CACHE}/torrents/{ID}.{INDEXER}.torrent`

> [!TIP]
> You can delete the cached `.torrent` files at any time. The application will just download them again if required.

The `queue` and `batch` commands will read and write the source statues to `{CACHE}/queue/{FIRST_BYTE_OF_HASH}.yml`

> [!WARNING]
> In theory you can delete the `cache/queue` files as they can be re-created using `queue add` however:
> - subsequent `batch` will be slow as it will need to re-process everything from scratch making an unnecessary number of I/O and API calls
> - `queue summary` will no longer include your uploads. Instead `verify` will just see them as all formats being transcoded already.
    > It's therefore recommended to leave these files alone.

> [!TIP]
> The `cache/queue` can be checked into version control. It uses a flat file format so changes can easily be tracked, backed up, and even reverted using `git`.

### Output Directory

The `spectrogram` command will generate spectrograms inside to
`{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} SPECTROGRAMS]/`

> [!TIP]
> Once you've reviewed the spectrograms you can freely delete each sectrograms directory (it can always be re-generated).

The `transcode` command will transcode to
`{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} {FORMAT}]/`

> [!TIP]
> You can delete each transcode directory if you:
> - Store the transcode elsewhere for seeding
> - Don't intend to produce transcodes or cross seed to another indexer.

Then `transcode` will create two `.torrent` files:
- `{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} {FORMAT}].{INDEXER}.torrent`
- `{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} {FORMAT}].torrent`

> [!TIP]
> You can delete the `.torrent` files if you:
> - Have already uploaded to the indexer
> - Don't intend to produce transcodes or cross seed to another indexer.

## Commands and Configuration

> [!TIP]
> **[Configuration options and the commands they apply to are documented in CONFIG.md](CONFIG.md)**

Configuration options are sourced first from the command line arguments, then from a configuration file.

By default the application loads `config.yml` from the current working directory, but this can be overridden with the `--config <CONFIG_PATH>` cli argument.

Most options have sensible defaults so the minimum required configuration is:

```yaml
announce_url: https://flacsfor.me/YOUR_ANNOUNCE_KEY/announce
api_key: "YOUR_API_KEY"
```

### Recommended configuration

This is based around the setup in this guide: [how to set up Deluge via Proton VPN with port forwarding](https://github.com/RogueOneEcho/how-to-setup-deluge-with-protonvpn-portforward).

#### Directory structure

- `/srv/shared` is a shared between multiple containers, by mounting as a single volume hard linking is possible.
- `/srv/deluge/state` is the Deluge state directory, containing all `.torrent` files loaded in Deluge.
- `/srv/shared/deluge` is the Deluge download directory, containing all the content.

#### `config.yml`

- `source: /srv/deluge/state,` in `config.yml` means the source can be ommitted from the command.

```yaml
announce_url: https://flacsfor.me/YOUR_ANNOUNCE_KEY/announce
api_key: YOUR_API_KEY
content:
- /srv/shared/deluge
limit: 2
output: /srv/shared/caesura
source: /srv/deluge/state
verbosity: debug
```

#### `docker-compose.yml`

- `user: "1000:1001"` ensures files have the same ownership as the host user (use the `id` command to find your user and group id).
- Only `/srv/shared` has write permissions, the other directories are read-only.
- `command: batch` runs the batch command by default.
- `/` is the working directory of the container so mounting the config to `/config.yml` means it's read by default.

```yaml
services:

  caesura:
    container_name: caesura
    image: ghcr.io/rogueoneecho/caesura
    user: "1000:1001"
    volumes:
    - /srv/caesura/config.yml:/config.yml:ro
    - /srv/deluge/state:/srv/deluge/state:ro
    - /srv/shared:/srv/shared
```

## Analyzing the queue

The `cache/queue` uses a YAML file format that can be analyzed with `yq`.

Filter` to see what has been transcoded:

```bash
cat ./cache/queue/*.yml | yq 'map(select(.transcode != null))'
```

Or to see what has been skipped and why:

```bash
cat ./cache/queue/*.yml | yq 'map(select(.verify.verified == false))'
```

If you're working with a lot of files then `less` can be helpful:

```bash
cat ./cache/queue/*.yml | yq --colors  'map(select(.verify.verified == false)) | less -R
```

## Troubleshooting

If you encounter any issues:

1. Check the logs for errors

The logging verbosity can be adjusted with the `--verbosity <LOG-LEVEL>` option. The available log levels are:

- `warn` only showing warnings and errors
- `info` will give an overview of what's happening
- `debug` provides insight into each step
- `trace` is detailed logging to see exactly what's happening

2. Ask ChatGPT

You might be surprised how often just copying and pasting the command and error message into ChatGPT can provide an instant solution.

3. Re-read the getting started guide
4. [Ask for help in support discussion](https://github.com/RogueOneEcho/caesura/discussions/categories/support)
5. If it's an idea or request for a new feature [search for an existing or create a new idea discussion](https://github.com/RogueOneEcho/caesura/discussions/categories/ideas)
5. If it's a bug report [search for an existing or create a new issue](https://github.com/RogueOneEcho/caesura/issues)


> [!TIP]
> If you manage to resolve your issue it's always worth creating a new discussion anyway because it might help someone else in the future, or identify an area where the documentation could be improved.

## Build

**[The build process is documented in BUILD.md](BUILD.md)**

## Releases and Changes

Releases and a full changelog are available via [GitHub Releases](https://github.com/RogueOneEcho/caesura/releases).

Release versions follow the [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) specification.

Commit messages follow the [Conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## History

[**DevYukine**](https://github.com/DevYukine) completed the **initial work** and released it as [**red_oxide**](https://github.com/DevYukine/red_oxide) under an [MIT license](LICENSE.HISTORIC.md).

[**RogueOneEcho**](https://github.com/RogueOneEcho) forked the project to complete a major refactor, **fix some issues**, add **new features** and improve logging and error handling. The fork is released as [**caesura**](https://github.com/RogueOneEcho/caesura) under an [AGPL license](LICENSE.md).

*The main difference between the former MIT license and the present AGPL license is that if you intend to distribute a modified version of the code - even to run it on a server - you must also provide the modified source code under an AGPL license.*

*This is often known as copyleft. The intent is to ensure that anyone taking advantage of this open source work are also contributing back to the open source community.*

The code base has now adopted [object oriented patterns](https://refactoring.guru/design-patterns/catalog) with SOLID principles and [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection).

See also the list of
[contributors](https://github.com/RogueOneEcho/caesura/contributors)
who participated in this project.
