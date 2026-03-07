# Docker Installation (recommended)

Docker is the recommended way to run caesura across all platforms.
- All dependencies are built into the image
- Runs in an isolated environment reducing risks to your system

## 1. Install Docker

[Install Docker Engine](https://docs.docker.com/engine/install/) for your OS.

## 2. Pull the image

```bash
docker pull ghcr.io/rogueoneecho/caesura
```

## 3. Set up `docker-compose.yml`

The repository includes a ready-to-use [`docker-compose.yml`](../docker-compose.yml) with security hardening and inline comments explaining each setting.

Copy it to your working directory and edit the volumes, config values, and user `UID:GID` to match your setup:

```bash
curl -O https://raw.githubusercontent.com/RogueOneEcho/caesura/main/docker-compose.yml
```

> [!TIP]
> The image runs as a non-root user by default so you will need update `user: "1000:1000"` to your `UID:GID` pair which can be found with:
>
> ```bash
> echo "$(id -u):$(id -g)"
> ```

The `docker-compose.yml` makes a few assumptions:
- `/srv/caesura/cache` will be your caesura cache
- `/srv/shared/downloads/content` is where your torrent content is stored
- `/srv/shared/caesura` will be where caesura outputs spectrograms, transcodes and `.torrent`
- `/srv/shared/caesura/autoadd` is where caesura will copy torrent files and your torrent client will autoadd from
- `/srv/qBittorrent/BT_backup` is where qBittorrent stores its.torrent files
- `/srv/shared/` contains the content and output directories. By mounting this as a single volume we can hard-link the content to the output directory.

If your system differs then you'll need to adjust the paths accordingly.

Refer to the [setup guide](SETUP.md) for more information on configuring caesura.

## 4. Running commands

The [command guide](COMMANDS.md) shows commands in native form. To run them with Docker Compose, prefix with `docker compose run --rm caesura`.

For example, the native version command is:

```bash
caesura version
```

For docker compose use:

```bash
docker compose run --rm caesura version
```

---

**Next: [Set up your configuration &rarr;](SETUP.md)**
