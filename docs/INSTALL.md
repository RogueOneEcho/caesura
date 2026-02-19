# Installation

The following methods are supported and are listed in order of recommendation.

## Docker (Linux, macOS, and Windows)

Refer to the [Docker guide](DOCKER.md).

## Homebrew (Linux and macOS)

1. [Install Homebrew](https://brew.sh/)

2. Install `caesura` from the [tap](https://github.com/RogueOneEcho/homebrew-tap).

This will install all the necessary dependencies and then download the correct binary of caesura from GitHub Releases for your system.

```bash
brew install rogueoneecho/tap/caesura
```

## Nix (Linux and macOS)

1. [Install Nix package manager](https://nixos.org/download/)

2. [Enable flakes](https://nixos.wiki/wiki/Flakes)

3. Install Caesura from the [flake](https://github.com/RogueOneEcho/nix).

```bash
nix profile install github:RogueOneEcho/nix#caesura
```

## Cargo (Linux and macOS)

*Cargo isn't recommended because you have to manually install the dependencies and these are unlikely to be as up to date or compatible as the brew or nix packages.*

1. Install the `flac`, `lame` and `sox_ng` with your preferred package manager or ideally build them from source.
- Sox: https://codeberg.org/sox_ng/sox_ng/#compiling-it-from-source
- FLAC: https://github.com/xiph/flac?tab=readme-ov-file#building-flac
- LAME: https://lame.sourceforge.io/download.php

2. Install `caesura` with Cargo.

```bash
cargo install caesura
```

## Windows

Windows currently isn't supported. Either use WSL or Docker.

Add a comment to [this discussion](https://github.com/RogueOneEcho/caesura/discussions/175) if you'd like to see Windows support.

## Build from source

Refer to the [build guide](BUILD.md).

---

**Next: [Set up your configuration &rarr;](SETUP.md)**
