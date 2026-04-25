# Setup

> [!TIP]
> Configuration options are sourced first from the command line arguments, then from a configuration file.

> [!TIP]
> By default the application loads `~/.config/caesura/config.yml` (or the platform equivalent), but this can be overridden with the `--config <CONFIG_PATH>` cli argument.

## Initial Configuration

Create a `config.yml` file with the following content:

- `announce_url` Your personal announce URL. Find it on the upload page.
- `api_key` Create an API key with `Torrents` permission: `Settings > Access Settings > Create an API Key`

```yaml
announce_url: https://flacsfor.me/YOUR_ANNOUNCE_KEY/announce
api_key: "YOUR_API_KEY"
```

> [!TIP]
> Refer to the [options reference](OPTIONS.md) for full documentation.

## Default paths

Default paths vary by platform:

| Key       | Docker        | Linux                                 | Windows                          | macOS                                              |
|-----------|---------------|---------------------------------------|----------------------------------|----------------------------------------------------|
| `config`  | `/config.yml` | `$XDG_CONFIG_HOME/caesura/config.yml` | `%APPDATA%/caesura/config.yml`   | `~/Library/Application Support/caesura/config.yml` |
| `cache`   | `/cache`      | `$XDG_CACHE_HOME/caesura/`            | `%LOCALAPPDATA%/caesura/`        | `~/Library/Caches/caesura/`                        |
| `output`  | `/output`     | `$XDG_DATA_HOME/caesura/output/`      | `%LOCALAPPDATA%/caesura/output/` | `~/Library/Application Support/caesura/output/`    |
| `reports` | `/output/reports` | `$XDG_DATA_HOME/caesura/output/reports/` | `%LOCALAPPDATA%/caesura/output/reports/` | `~/Library/Application Support/caesura/output/reports/` |
| `content` | `/content`    | -                                     | -                                | -                                                  |

`content` must be set explicitly for native installations.

If your system differs from this you will need to set the appropriate paths in your config file.

> [!TIP]
> Refer to the [directories guide](DIRECTORIES.md) for a complete explanation of the files that `caesura` creates.

### Verify your configuration

Run the `config` command to see the full resolved configuration including default values:

```bash
caesura config
```

## Recommended config

`caesura` does adopt sensible defaults for most options but for an optimal experience the following is recommended:

```yaml
# Announce URL including passkey
# Examples: `https://flacsfor.me/a1b2c3d4e5f6/announce`, `https://home.opsfet.ch/a1b2c3d4e5f6/announce`
# Default: ""
announce_url: https://tracker.example.com/YOUR_ANNOUNCE_KEY/announce
# API key with torrent permissions for the indexer.
# Default: ""
api_key: YOUR_API_KEY
# Directories containing torrent content.
# Typically this is set as the download directory in your torrent client.
# Default: []
content:
- /srv/shared/downloads/content
- /srv/shared/some/other/dir
- /srv/shared/some/other/dir/sub/path
- /home/user/Music
# Directory the torrent file is copied to.
# This should be set if you wish to auto-add to your torrent client.
copy_torrent_to: /srv/shared/caesura/autoadd
# Should files be hard linked instead of copied?
# Enabling this option requires the source and destination to be on the same filesystem or mounted volume.
# Default: false
hard_link: true
# Directory where transcodes and spectrograms will be written.
# Default: `~/.local/share/caesura/output/` or platform equivalent
output: /srv/shared/caesura
# A path to either a directory of `.torrent` files or a single YAML queue file.
# If you set this to the directory your torrent client stores `.torrent` files then caesura
# will automatically load everything from your client.
# - For qBittorrent use the `BT_backup` directory
# - For deluge use the `state` directory
# Examples:
# - `/srv/qBittorrent/BT_backup`
# - `/srv/deluge/state`
# - `./queue.yml`
queue_add_path: /srv/qBittorrent/BT_backup
# Should transcoded files be renamed?
# If enabled then tracks are renamed into a standardized format: `{number} {title}.{ext}`.
# Multi-disc releases will be organized into `CD1/`, `CD2/` subfolders.
# - `1 Example track title.flac`
# - `CD1/10 Example track title.mp3`
# Default: false
rename_tracks: true
# Is `SoX_ng` in use?
# If `true` then `sox_ng` specific CLI options are used.
# Default: Detected based on binary name or --version info
sox_ng: true
# Name or path to the sox binary.
# Examples: `sox`, `sox_ng`, `/usr/bin/sox`
# Default: Detected based on sox_ng flag
sox_path: sox_ng
# Formats to attempt to transcode to.
# Default: ["flac","320","v0"]
target:
- flac
- v0
- '320'
# Level of logs to display.
# Default: "info"
verbosity: debug
```

## Torrent client integration

The qui or qBittorrent API can be used for queuing and injecting torrents.

### qBittorrent via qui (recommended)

[qui](https://getqui.com/) has a [reverse proxy feature](https://getqui.com/docs/features/reverse-proxy/) with caching that makes qBittorrent API calls faster.

Point `qbit_url` at your qui instance's [Client Proxy URL](https://getqui.com/docs/features/reverse-proxy/#1-create-a-client-proxy-api-key), which includes the proxy API key directly in the path:

```yaml
inject_torrent: true
qbit_url: http://localhost:7476/proxy/YOUR_QUI_API_KEY
qbit_fetch_categories:
- music
qbit_inject_category: caesura
qbit_inject_tags:
- caesura
```

### qBittorrent API directly

If you don't use qui, point `qbit_url` at qBittorrent's Web UI directly and set `qbit_username` and `qbit_password`.

```yaml
inject_torrent: true
qbit_url: http://127.0.0.1:8080
qbit_username: YOUR_QBIT_USERNAME
qbit_password: YOUR_QBIT_PASSWORD
qbit_fetch_categories:
- music
qbit_inject_category: caesura
qbit_inject_tags:
- caesura
```

This is slower than the qui route because every call hits qBittorrent with no caching, but still faster than file-based discovery.

### Autoadd / watch directory

For Deluge, Transmission, rTorrent, or anyone who prefers a file-based integration. `caesura` can:
- write the transcoded `.torrent` to a watch directory your client monitors
- read source torrents from your client's torrent file directory

```yaml
copy_torrent_to: /srv/shared/caesura/autoadd
queue_add_path: /srv/qBittorrent/BT_backup
```

If using Docker then ensure the autoadd and source torrent directories are mounted as volumes in `docker-compose.yml`.

## Multi-Indexer Setup (RED + OPS)

`caesura` is designed to work with both `RED` and `OPS`. There's no need for separate cache or output directories, however, you will need a separate configuration for each and the commands must be run separately.

Make a copy of `config.yml` for each indexer. For clarity I recommend naming them `config.red.yml` and `config.ops.yml`

Edit each config file to include the API key and announce URL for that indexer.

```yaml
announce_url: https://home.opsfet.ch/YOUR_ANNOUNCE_KEY/announce
api_key: "YOUR_API_KEY"
```

If you're using docker then edit `docker-compose.yml` to include separate services for each indexer. The only difference between them is the mapping of the config file.

> [!NOTE]
> If you start a transcode for a source on OPS that you've already transcoded for RED then `caesura` will detect this automatically and instead of re-transcoding it simply creates a `*.ops.torrent` file from the existing transcode so there's no duplication of effort and the existing files are re-used without taking up additional space.
>
> Therefore the first time you run the `batch` command for the new indexer you will likely see a few messages along the lines of:
>
> ```
> Found existing 320 transcode
> Found existing V0 transcode
> ```

---

**Next: [Command Guide &rarr;](COMMANDS.md)**
