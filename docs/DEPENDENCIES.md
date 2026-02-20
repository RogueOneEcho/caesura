# Dependencies

The official repository for [SoX](https://sourceforge.net/projects/sox/) is no longer maintained and has a significant number of known issues. Most distributions have patched versions of SoX available but there is very little consistency. The [SoX_ng project](https://codeberg.org/sox_ng/sox_ng) has emerged as a likely successor, unifying the various patches and committing to regular releases.

`caesura` will continue to work with regular `sox` but it is recommended to use `sox_ng` for the best experience.

| Tool   | Recommended Version | Purpose                                                     |
|--------|---------------------|-------------------------------------------------------------|
| SoX_ng | `14.7.0.9`          | Resampling hi-res audio, spectrogram generation             |
| FLAC   | `1.5.0`             | Decoding FLAC to WAV for MP3 encoding, encoding WAV to FLAC |
| LAME   | `3.100`             | Encoding MP3 (V0 and 320 CBR)                               |

## Install

| Platform        | Install                          | Includes `SoX_ng` | Note                                                                  |
|-----------------|----------------------------------|-------------------|-----------------------------------------------------------------------|
| Homebrew        | `brew install sox_ng flac lame`  | ✅                |                                                                       |
| Arch Linux      | `pacman -S sox flac lame`        | ✅                | [sox is sox_ng](https://archlinux.org/packages/extra/x86_64/sox/)     |
| Gentoo          | `emerge sox flac lame`           | ✅                | [sox is sox_ng](https://packages.gentoo.org/packages/media-sound/sox) |
| Debian / Ubuntu | `apt install sox flac lame`      | ❌                |                                                                       |
| Fedora          | `dnf install sox flac lame`      | ❌                |                                                                       |
| Nix             | `nix-shell -p sox flac lame`     | ❌                |                                                                       |

## SoX_ng

If you wish to use SoX_ng and your distribution does not provide it then try the following:

### Build SoX_ng from source

Follow the [official instructions](https://codeberg.org/sox_ng/sox_ng/#compiling-it-from-source)

### Install Prebuilt SoX_ng Binaries

SoX_ng depends on libFLAC at build time, so it should be built against your distribution's FLAC package.

If you're unable to build it yourself then you can try these experimental binaries which are built specifically for `caesura`.

> [!CAUTION]
> These are experimental and may not work with other tools.

1. Download the latest archive for your platform from: https://github.com/RogueOneEcho/install/releases

| OS    | CPU           | Binary Suffix                | Note                     |
|-------|---------------|------------------------------|--------------------------|
| Linux | Intel/AMD     | `x86_64-unknown-linux-gnu`   | Standard (with glibc)    |
| Linux | ARM           | `aarch64-unknown-linux-gnu`  | Standard (with glibc)    |
| Linux | Intel/AMD     | `x86_64-unknown-linux-musl`  | Portable (without glibc) |
| Linux | ARM           | `aarch64-unknown-linux-musl` | Portable (without glibc) |
| macOS | Apple Silicon | `aarch64-apple-darwin`       |                          |
| macOS | Intel         | `x86_64-apple-darwin`        |                          |

```bash
curl -fSL "https://github.com/RogueOneEcho/install/releases/latest/download/sox_ng-x86_64-unknown-linux-musl.tar.xz" -o sox_ng.tar.xz
```

2. Extract `sox_ng` and `flac` from the archive and add them to your PATH.

```bash
tar -xJf sox_ng.tar.xz -C ~/.local/bin sox_ng flac
```

3. Make them executable.

```bash
chmod +x ~/.local/bin/sox_ng ~/.local/bin/flac
```

4. Verify they work

```bash
flac --version
sox_ng --version
```
