# Updating Dependencies

SoX_ng, FLAC and LAME versions are pinned in several places.

They should be updated together so everything stays in sync.

## Version

CI installs SoX_ng, FLAC and LAME from Homebrew, so the pinned versions should match Homebrew.

Check the current Homebrew versions:

```bash
brew info sox_ng flac lame
```

The [Homebrew tap](https://github.com/RogueOneEcho/homebrew-tap) installs from Homebrew, so it needs no changes.

## caesura

| File                                | Change                                   |
|-------------------------------------|------------------------------------------|
| `Dockerfile`                        | `*_VERSION` and `*_SHA256` build args    |
| `.github/sbom/source-deps.cdx.json` | `version`, `purl` and `distribution` URL |
| `docs/DEPENDENCIES.md`              | Recommended version table                |
| `docs/TESTING.md`                   | Required tools table                     |

Compute the SHA256 of the archive downloaded in the `Dockerfile`:

```bash
curl -fsSL "<archive url>" | sha256sum
```

Clear the sample cache and run the tests. Snapshots may change with a new version:

```bash
./samples/rm-samples && cargo test --quiet --all-features
```

## Prebuilt SoX_ng binaries

The [install](https://github.com/RogueOneEcho/install) repo builds the [prebuilt binaries](DEPENDENCIES.md#install-prebuilt-sox_ng-binaries).

1. Update `SOX_NG_VERSION` or `FLAC_VERSION` in `.github/workflows/build-sox-ng.yml` and push
2. Publish a release:

```bash
gh workflow run build-sox-ng.yml -R RogueOneEcho/install -f sox_ng_version=<version> -f flac_version=<version>
```

## Nix

The [nix](https://github.com/RogueOneEcho/nix) repo packages SoX_ng. FLAC and LAME come from nixpkgs.

```bash
scripts/update-sox-ng.sh <version>
```
