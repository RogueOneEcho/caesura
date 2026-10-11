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

## GitHub Releases (Linux, macOS, and Windows)

1. [Install the dependencies](DEPENDENCIES.md)

2. Download the archive for your platform from the [latest release](https://github.com/RogueOneEcho/caesura/releases/latest).

| OS      | CPU           | Binary Suffix                | Note                     |
|---------|---------------|------------------------------|--------------------------|
| Linux   | Intel/AMD     | `x86_64-unknown-linux-gnu`   | Standard (with glibc)    |
| Linux   | ARM           | `aarch64-unknown-linux-gnu`  | Standard (with glibc)    |
| Linux   | Intel/AMD     | `x86_64-unknown-linux-musl`  | Portable (without glibc) |
| Linux   | ARM           | `aarch64-unknown-linux-musl` | Portable (without glibc) |
| macOS   | Apple Silicon | `aarch64-apple-darwin`       |                          |
| Windows | Intel/AMD     | `x86_64-pc-windows-msvc`     | `.zip` archive           |

3. Extract the binary and move it to a directory on your `PATH`.

On Linux and macOS:

```bash
tar -xJf caesura-*.tar.xz
mv caesura /usr/local/bin/caesura
```

On Windows, extract `caesura.exe` from the `.zip` and move it to a folder on your `PATH`.

### Install script (Linux and macOS)

Alternatively, on Linux and macOS, use the install script to automate the above steps:

> [!CAUTION]
> This script is experimental and piping straight to bash can be dangerous.
>
> Only proceed if you understand the risks and are running on an isolated, recoverable system without sensitive data.

```bash
curl -fsSL https://raw.githubusercontent.com/RogueOneEcho/install/main/scripts/install-caesura.sh | bash
```

## Cargo (Linux, macOS, and Windows)

*Cargo isn't recommended because you have to manually install the dependencies and these are unlikely to be as up to date or compatible as the brew or nix packages.*

1. [Install the dependencies](DEPENDENCIES.md)

2. Install `caesura` with Cargo.

```bash
cargo install caesura
```

## Build from source

Refer to the [build guide](BUILD.md).

---

**Next: [Set up your configuration &rarr;](SETUP.md)**
