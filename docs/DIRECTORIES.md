## Directories Guide

The application requires two writable directories.

### Cache Directory

The `verify` command will download `.torrent` files for each source to `{CACHE}/torrents/{ID}.{INDEXER}.torrent`

> [!TIP]
> You can delete the cached `.torrent` files at any time. The application will just download them again if required.

The `queue` and `batch` commands will read and write the source statuses to `{CACHE}/queue/{FIRST_BYTE_OF_HASH}.yml`

> [!WARNING]
> In theory you can delete the `cache/queue` files as they can be re-created using `queue add` however:
> - subsequent `batch` will be slow as it will need to re-process everything from scratch making an unnecessary number of I/O and API calls
> - `queue summary` will no longer include your uploads. Instead `verify` will just see them as all formats being transcoded already.
    >   It's therefore recommended to leave these files alone.

> [!TIP]
> The `cache/queue` can be checked into version control. It uses a flat file format so changes can easily be tracked, backed up, and even reverted using `git`.

### Output Directory

The `spectrogram` command will generate spectrograms inside
`{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} SPECTROGRAMS]/`

> [!TIP]
> Once you've reviewed the spectrograms you can freely delete each spectrograms directory (it can always be re-generated).

The `transcode` command will transcode to
`{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} {FORMAT}]/`

> [!TIP]
> You can delete each transcode directory if you:
> - Store the transcode elsewhere for seeding
> - Don't intend to produce transcodes or cross seed to another indexer.

Then `transcode` will create a `.torrent` files:
- `{OUTPUT}/{ARTIST} - {ALBUM} [{YEAR}] [{MEDIA} {FORMAT}].{INDEXER}.torrent`

> [!TIP]
> You can delete the `.torrent` files if you:
> - Have already uploaded to the indexer
> - Don't intend to produce transcodes or cross seed to another indexer.
