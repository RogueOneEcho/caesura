# Command Guide

> [!TIP]
> You can append `--help` to any command to see the available options.
>
> **[CLI flags and `config.yml` file options are documented in the options reference](OPTIONS.md)**

## `version`

Display version and dependency info.

```bash
caesura version
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/version.gif)

If any dependencies are missing they will be clearly indicated.

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/missing-dependency.gif)

## `config`

Show resolved configuration with defaults.

```bash
caesura config
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/config.gif)

## `inspect`

The inspect command can be used on any directory of `.flac` or `.mp3` files.

It prints a table of metadata for each file followed by printing every tag and embedded image in the file.

It's useful for checking that the metadata of a source is correct before transcoding.

```bash
caesura inspect "/path/to/Artist - Album"
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/inspect.gif)

## `verify`

Verify a FLAC source is suitable for transcoding.

```bash
caesura verify 142659
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/verify.gif)

> [!TIP]
> For the source you can use a permalink, the numeric torrent id, a path to a torrent file, or a 40-character info hash.
> Examples:
> - `caesura verify https://redacted.sh/torrents.php?id=80518&torrentid=142659#torrent142659`
> - `caesura verify 142659`
> - `caesura verify "../path/to/Artist - Hello World [2024].torrent"`
> - `caesura verify 0123456789abcdef0123456789abcdef01234567`

If it looks good you can proceed to transcoding, otherwise try another source.

## `spectrogram`

Generate full and zoomed spectrograms for review.

```bash
caesura spectrogram 142659
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/spectrogram.gif)

Inspect the spectrograms in the output directory.

## `transcode`

Transcode FLAC to target formats.

```bash
caesura transcode 142659
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/transcode.gif)

Inspect the transcodes in the output directory.

> [!TIP]
> Things to check:
> - Folder structure
> - File names
> - Tags
> - Audio quality
> - Image size and compression quality

## `upload`

Upload transcodes to your indexer.

> [!WARNING]
> You are responsible for everything you upload.
>
> Misuse of this application can result in the loss of your upload privileges.

> [!TIP]
> If you're unsure about this then you can append `--dry-run` to the command and instead of uploading it will print the data that would be submitted.

```bash
caesura upload 142659 --dry-run
```

```bash
caesura upload 142659
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/upload.gif)

If you haven't already then add the `*.red.torrent` or `*.ops.torrent` file to your torrent client.

> [!TIP]
> `caesura` can automatically add the transcoded `.torrent` to your client: see [Torrent client integration](SETUP.md#torrent-client-integration).

Go to your indexer and check your uploads to make sure everything has gone to plan.

## Batch processing with queue management

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/batch.gif)

> [!WARNING]
> You are responsible for everything you upload.
>
> Misuse of this application, especially the `batch` command, can result in the loss of your upload privileges or a ban.

The `batch` command handles `verify`, `spectrogram`, `transcode` and `upload` in a single command.

### `queue fetch`

If you use qBittorrent the `queue fetch` command discovers fully downloaded torrents via the qBittorrent API and adds them to the queue.

> [!TIP]
> Refer to [Torrent client integration](SETUP.md#torrent-client-integration) for setup instructions.

```bash
caesura queue fetch
```

### `queue add`

If you don't use qBittorrent the `queue add` command adds all `.torrent` files from a directory to the queue.

If you pass the directory your torrent client stores `.torrent` files then caesura will automatically load everything from your client.

- For qBittorrent use the `BT_backup` directory
- For deluge use the `state` directory

```bash
caesura queue add /path/to/your/torrents
```

> [!TIP]
> Instead of including the path as an argument you can set it in the config file:
> ```yaml
> queue_add_path: /srv/qBittorrent/BT_backup
> ```
> Then run the command as:
>
> ```bash
> caesura queue add
> ```

### `queue list`

List what is next in the queue for the current `indexer`:

```bash
caesura queue list
```

### `queue rm`

Remove items from the queue:

```bash
caesura queue rm <HASH>
```

### `queue summary`

View queue progress summary:

```bash
caesura queue summary
```

### `batch`

By default the `batch` command will limit to processing just `3` transcodes and it won't create spectrograms or upload unless explicitly instructed. These safeguards are in place to prevent mistakenly uploading a bunch of sources that you haven't checked.

**Verify and transcode 3 sources:**

```bash
caesura batch --transcode
```

> [!TIP]
> Add the `--spectrogram` flag to generate spectrograms.

**Upload the transcodes you've already checked:**

```bash
caesura batch --transcode --upload
```

**Transcode everything (but don't upload):**

```bash
caesura batch --transcode --no-limit
```

**Upload in batches with a wait interval:**

```bash
caesura batch --upload --limit 10 --wait-before-upload 30s
```

> [!WARNING]
> In theory you can execute with both `--upload --no-limit` but that is probably a bad idea and a very fast way to lose your upload privileges.
>
> If you are going to do so then you should definitely use a long wait interval:
> `--upload --no-limit --wait-before-upload 2m`

### Analyzing the Queue

The `cache/queue` uses a YAML file format that can be analyzed with `yq`.

Filter to see what has been transcoded:

```bash
cat ./cache/queue/*.yml | yq 'map(select(.transcode != null))'
```

Or to see what has been skipped and why:

```bash
cat ./cache/queue/*.yml | yq 'map(select(.verify.verified == false))'
```

If you're working with a lot of files then `less` can be helpful:

```bash
cat ./cache/queue/*.yml | yq --colors 'map(select(.verify.verified == false))' | less -R
```

## `audit`

Scan a directory of `.torrent` files for problematic file paths.

```bash
caesura audit path/to/torrents
```

![](https://media.githubusercontent.com/media/RogueOneEcho/assets-caesura/main/dist/audit.gif)

If you point it at the directory your torrent client stores `.torrent` files then caesura will scan everything from your client.

- For qBittorrent use the `BT_backup` directory
- For deluge use the `state` directory

The command also accepts a torrent id. The respective torrent is then downloaded from the indexer API.

This requires `api_key` and `indexer_url` to be set in the config file.

```bash
caesura audit 12345
```

> [!WARNING]
> While audit issues can be problematic they aren't necessarily rule breaking. Check before reporting.

> [!TIP]
> By default it runs every check. Disable individual checks with the `--ignore-*` flags:
>
> - `--ignore-non-utf8` - paths that are not valid UTF-8
> - `--ignore-single-file` - single-file torrents
> - `--ignore-libtorrent` - characters libtorrent strips on disk
> - `--ignore-invisible` - invisible or zero-width characters
> - `--ignore-directional` - unnecessary directional marks
> - `--ignore-unsafe` - unsafe path segments
> - `--ignore-nfd` - decomposed (non-NFC) characters
> - `--ignore-broken-extension` - file extensions broken by libtorrent
> - `--ignore-leading-period` - path components with a leading period
> - `--ignore-leading-space` - path components with a leading space
> - `--ignore-trailing-space` - path components with a trailing space
>
> ```bash
> caesura audit path/to/torrents --ignore-invisible --ignore-libtorrent --ignore-unsafe --ignore-nfd --ignore-single-file --ignore-non-utf8
> ```

> [!TIP]
> Add `--print-bb-code` to render the diffs with BB code, ready to paste into a tracker report.

```bash
caesura audit /srv/torrents --print-bb-code
```

## `cross`

Cross-seed a source onto a second indexer.

> [!NOTE]
> `cross` exists to support an unreleased feature and isn't much use on its own.
>
> It uses similar mechanics to [qui's cross-seeding](https://getqui.com/docs/features/cross-seed/gazelle-ops-red/), which is the recommended approach.

Given a source, `cross` looks up whether the same release exists on another indexer (the "cross" indexer). If a match is found it downloads the cross-seed `.torrent` and adds it to your torrent client so you seed the same files on both.

The cross indexer is configured with a separate config file passed via `--cross-config`. Only `api_key`, `indexer`, and `indexer_url` are read from it.

> [!TIP]
> Append `--dry-run` to look up the cross-seed without downloading or injecting.

You must set at least one of:

- `--qbit-cross` to inject the cross-seed into qBittorrent
- `--copy-cross-torrent-to <DIR>` to copy the `.torrent` into a watch directory
- `--dry-run` to only report the match

```bash
caesura cross 142659 --cross-config ops.yml --qbit-cross
```

> [!TIP]
> `--qbit-cross` uses your existing qBittorrent connection: see [Torrent client integration](SETUP.md#torrent-client-integration).
>
> Injected cross-seeds default to the `caesura` category and tag. Override placement with `--qbit-cross-category`, `--qbit-cross-tags`, `--qbit-cross-savepath`, `--qbit-cross-paused`, or `--qbit-cross-skip-checking`.

## Troubleshooting

Refer to the [troubleshooting guide](TROUBLESHOOTING.md) if you have any issues.

---

**Next: [Options Reference &rarr;](OPTIONS.md)**
