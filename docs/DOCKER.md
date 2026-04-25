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
- `/srv/shared/` contains the content and output directories. By mounting this as a single volume we can hard-link the content to the output directory.
- qui is used as a [reverse proxy](https://getqui.com/docs/features/reverse-proxy/) for qBittorrent

If your system differs then you'll need to adjust the paths or [torrent client integration](SETUP.md#torrent-client-integration) accordingly.

Refer to the [setup guide](SETUP.md) for more information on configuring caesura.

## 4. Running commands

The [command guide](COMMANDS.md) shows commands in native form. To run them with Docker Compose, prefix with `docker compose run --rm caesura`.

Any path used in the config file or passed to a command must be the path as seen inside the container. In the example compose file, `/srv/shared` is mounted at the same path inside the container, so values such as `/srv/shared/downloads/content` work in both places. If you mount a host directory somewhere else in the container, use the container-side path.

For example, the native version command is:

```bash
caesura version
```

For docker compose use:

```bash
docker compose run --rm caesura version
```

The same rule applies to command arguments. To inspect an album directory mounted by the example compose file:

```bash
docker compose run --rm caesura inspect "/srv/shared/downloads/content/Artist - Album"
```

---

**Next: [Set up your configuration &rarr;](SETUP.md)**
