## Build

The getting started guide uses the pre-built Docker image but for increase privacy and assurance of what you're running you can easily build the application yourself with cargo.

### Dependencies

First you'll need to install the dependencies for your OS.

#### Windows

Just use Docker or ask ChatGPT. I imagine it's tedious.

#### MacOS and Linux with Homebrew

1. [Install Rust](https://www.rust-lang.org/tools/install)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. [Install Intermodal](https://github.com/casey/intermodal#installation)

From Cargo:

```bash
cargo install imdl
```

Or, from GitHub Releases:

```bash
curl "https://github.com/casey/intermodal/releases/download/v0.1.14/imdl-v0.1.14-x86_64-unknown-linux-musl.tar.gz" \
  --location \
  --show-error \
  --silent \
| tar \
  --extract \
  --gzip \
  --directory "/usr/local/bin" \
  --file - \
  "imdl"
```

4. Install FLAC, LAME and SOX dependencies.

With Homebrew:

```bash
brew install flac lame sox eye-d3
```

Or, from your package manager:

```bash
sudo apt install flac lame sox --yes
```

5. MacOS Only

As an Apple user you'll be familiar that everything comes at a premium. So you should probably send me some bitcoin, ethereum, or monero before proceeding.


### Install with Cargo

```bash
cargo install caesura
```

### Build with Cargo

```bash
git clone https://github.com/RogueOneEcho/caesura.git
cd caesura
cargo run -- verify "123456"
```

### Build with Docker

Build and tag:

```bash
git clone https://github.com/RogueOneEcho/caesura.git
cd caesura
docker build -t caesura .
```

Run:

```bash
docker run \
-v ./config.json:/config.json \
-v /path/to/your/content:/content \
-v ./output:/output \
ghcr.io/rogueoneecho/caesura \
verify 142659
```

#### Docker Compose

Clone the repo and edit the volumes in `docker-compose.yml`.

```bash
git clone https://github.com/RogueOneEcho/caesura.git
cd caesura
nano docker-compose.yml
```

Single run:

```bash
docker compose run --rm caesura verify "123456"
```

Or, start up the services and follow the logs:

```bash
docker compose up -d caesura
docker logs -f caesura
```

Or, run the full suite including a [Caddy](https://caddyserver.com/) server to serve the output directory:

```bash
docker compose up -d
docker logs -f caesura
```
