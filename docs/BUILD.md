## Build from source

### Build with Cargo

1. Install the `flac`, `lame` and `sox_ng` with your preferred package manager or ideally build them from source.
- Sox: https://codeberg.org/sox_ng/sox_ng/#compiling-it-from-source
- FLAC: https://github.com/xiph/flac?tab=readme-ov-file#building-flac
- LAME: https://lame.sourceforge.io/download.php

2. Clone the repo

```bash
git clone https://github.com/RogueOneEcho/caesura.git
```

3. Enter the repo directory and build the project

```bash
cd caesura
```
```bash
cargo build
```

4. Run the tests

> [!NOTE]
> Tests should complete quickly. Expect ~45s on a CI runner and under 10s on a modern desktop. If it's taking longer this could indicate a dependency issue. See [TESTING.md](docs/TESTING.md) for full details on the test infrastructure.

```bash
cargo test --quiet --all-features
```

5. Run

```bash
cargo run -- version
```

### Build with Docker Compose

1. Clone the repo

```bash
git clone https://github.com/RogueOneEcho/caesura.git
```

2. Enter the repo directory

```bash
cd caesura
```

3. Replace `image: ghcr.io/rogueoneecho/caesura:latest` with `build: .` in `docker-compose.yml`.

```bash
sed -i 's|image: ghcr.io/rogueoneecho/caesura:latest|build: .|' docker-compose.yml
```

4. Configure the `docker-compose.yml` to match your environment.

5. Build the container

```bash
docker compose build caesura
```

6. Run the container

```bash
docker compose run --rm caesura version
```
