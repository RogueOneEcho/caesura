# Comparison

Comparison with related tools.


>[!NOTE]
> This comparison is best-effort.
> - It is likely to contain inaccuracies due to large number of other projects
> - It has a clear bias towards `caesura`.

>[!CAUTION]
> Just because a tool is listed here does NOT mean it is safe to use.
>
> None of these tools have been reviewed, audited, or used.
>
> This analysis is based on documentation only.

## Methodology

- **Stars, language, last activity, CI/CD**: from GitHub, GitLab, and Codeberg repository metadata.
- **Docker**: Search for `Dockerfile` or `docker-compose.*` files.
- **Tests**: Search for `tests/` or `test/` directories, or file naming conventions like `_test.rs`, `.test.js`, `test_*.py`.
- **Features**: Review the README.md and any linked documentation.
- **Platform**: Review installation docs and release assets (e.g. prebuilt binaries for Windows, macOS, Linux).
- **Sort order**: Rows are sorted by number of features (descending), then by stars (descending).
- **Last activity**: 🟢 < 6 months. 🟠 < 18 months. 🔴 > 18 months.
- **Staleness**: Star counts and activity dates were captured at the time of writing. Use the last commit date of this file to assess staleness.

## Transcoding Tools

| Repo                                                                                              | RED | OPS | [API](#api) | Verify | Spec | Upload | Batch | [CI](#cicd-and-tests) | [Tests](#cicd-and-tests) | [Platforms](#platforms) | [Lang](#language) | Stars | Active | Notes |
|---------------------------------------------------------------------------------------------------| :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | --- | ---: | --- | --- |
| [<small>RogueOneEcho/</small>​**caesura**](https://github.com/RogueOneEcho/caesura)               | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🍎 🐧 🐳 | 🟢 Rust | 156 | 🟢 2026 | |
| [<small>smokin-salmon/</small>​**smoked-salmon**](https://github.com/smokin-salmon/smoked-salmon) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | 🪟 🍎 🐧 🐳 | 🔴 Python | 148 | 🟢 2026 | Maintained fork with OPS, DIC, and API key support |
| [<small>DevYukine/</small>​**red_oxide**](https://github.com/DevYukine/red_oxide)                 | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | 🪟 🍎 🐧 | 🟢 Rust | 147 | 🟠 2024 | Predecessor project caesura was forked from |
| [<small>lfence/</small>​**red-trul**](https://github.com/lfence/red-trul)                         | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | 🪟 🍎 🐧 | 🟠 JavaScript | 48 | 🟠 2025 | Non-interactive transcoder and uploader for RED |
| [<small>walkrflocka/</small>​**orpheusmorebetter**](https://github.com/walkrflocka/orpheusmorebetter) | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | 🐧 | 🔴 Python | 23 | 🟢 2026 | Fork of whatbetter-crawler for Orpheus |
| [<small>RecursiveForest/</small>​**whatmp3**](https://github.com/RecursiveForest/whatmp3)          | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 233 | 🟠 2025 | Multi-format transcoder with mktorrent |
| [<small>beep_street/</small>​**downsampler-threaded**](https://gitlab.com/beep_street/downsampler-threaded) | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Shell | 4 | 🟠 2024 | Multi-process SoX frontend for FLAC downsampling |
| [<small>doujincafe/</small>​**reencoder**](https://github.com/doujincafe/reencoder)               | - | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | 🐧 | 🟢 Rust | 2 | 🟢 2025 | Batch FLAC re-encoder with SQLite tracking |
| [<small>flacquack/</small>​**flac50**](https://codeberg.org/flacquack/flac50)                     | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 0 | 🟢 2026 | Batch FLAC to MP3 V0/320 transcoder |
| [<small>whatloss/</small>​**whatloss**](https://codeberg.org/whatloss/whatloss)                   | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | 🐧 | 🟢 Rust | 2 | 🟠 2024 | Configurable transcoder with Tera-templated commands |

## Inactive Transcoding Tools

| Repo | RED | OPS | [API](#api) | Verify | Spec | Upload | Batch | [CI](#cicd-and-tests) | [Tests](#cicd-and-tests) | [Platforms](#platforms) | [Lang](#language) | Stars | Active | Notes |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |:---------------:| --- | ---: | --- | --- |
| [<small>ligh7s/</small>​**smoked-salmon**](https://github.com/ligh7s/smoked-salmon) | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |       🐧        | 🔴 Python | 145 | 🔴 2024 | Original smoked-salmon |
| [<small>stormgit/</small>​**red-better**](https://gitlab.com/stormgit/red-better) | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |      🍎 🐧      | 🔴 Python | 4 | 🔴 2022 | REDBetter fork with spectral/hashcheck verification |
| [<small>kpdean/</small>​**RED_OPS_Better**](https://github.com/kpdean/RED_OPS_Better) | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |       🐧        | 🔴 Python | 33 | 🔴 2021 | REDBetter fork with OPS support |
| [<small>iw00t/</small>​**REDBetter**](https://github.com/iw00t/REDBetter) | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ |    🍎 🐧 🐳     | 🔴 Python | 163 | 🔴 2023 | Widely forked RED transcoder and uploader |
| [<small>Mechazawa/</small>​**REDBetter-crawler**](https://github.com/Mechazawa/REDBetter-crawler) | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |      🍎 🐧      | 🔴 Python | 144 | 🔴 2023 | Autonomous variant of REDBetter |
| [<small>ApexWeed/</small>​**orpheusbetter-crawler**](https://github.com/ApexWeed/orpheusbetter-crawler) | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |       🐧        | 🔴 Python | 117 | 🔴 2022 | whatbetter fork for Orpheus |
| [<small>jtpavlock/</small>​**bettered**](https://github.com/jtpavlock/bettered) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |       🐧        | 🔴 Python | 5 | 🔴 2022 | Helper script for redacted better.php |
| [**pastebin**](https://pastebin.com/uBp9hhwC) | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |       🐧        | 🔴 Shell | - | - | Unnamed script for FLAC to MP3/16-bit with mktorrent |
| [<small>beep_street/</small>​**mkopuslibrary**](https://gitlab.com/beep_street/mkopuslibrary) | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |       🐧        | 🔴 Shell | 3 | 🔴 2024 | Multi-process frontend for opusenc |
| [<small>mahkitah/</small>​**m3ercat**](https://github.com/mahkitah/m3ercat) | - | - | - | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |       🪟        | 🔴 Python | 0 | 🔴 2024 | FLAC-to-MP3 transcoder with thorough tag preservation |

## Other Tools

| Repo | RED | OPS | [API](#api) | Verify | Spec | Upload | Batch | [CI](#cicd-and-tests) | [Tests](#cicd-and-tests) | [Platforms](#platforms) | [Lang](#language) | Stars | Active | Notes |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | --- | ---: | --- | --- |
| [<small>_mclovin/</small>​**redcurry**](https://gitlab.com/_mclovin/redcurry) | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | 🍎 🐧 | 🟠 Ruby | 18 | 🟢 2026 | Transfers torrents between Gazelle trackers |
| [<small>mahkitah/</small>​**Transplant**](https://github.com/mahkitah/Transplant) | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | 🪟 🐧 | 🔴 Python | 62 | 🟢 2026 | Cross-posts torrents between RED and OPS |
| [<small>_mclovin/</small>​**gazelle-specs**](https://gitlab.com/_mclovin/gazelle-specs) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🟠 Ruby | 5 | 🟢 2025 | Generates spectrograms and uploads to release desc |
| [<small>KyokoMiki/</small>​**nemorosa**](https://github.com/KyokoMiki/nemorosa) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | 🪟 🍎 🐧 🐳 | 🔴 Python | 74 | 🟢 2026 | Cross-seeder for music torrents |
| [<small>moleculekayak/</small>​**fertilizer**](https://github.com/moleculekayak/fertilizer) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | 🐧 🐳 | 🔴 Python | 118 | 🟠 2024 | Fork of crops; cross-seeder with client injection |
| [<small>soranosita/</small>​**crops**](https://github.com/soranosita/crops) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 35 | 🔴 2023 | Archived; cross-seeder predecessor to fertilizer |
| [<small>sirjaren/</small>​**redoflacs**](https://github.com/sirjaren/redoflacs) | - | - | - | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | 🍎 🐧 | 🔴 Shell | 116 | 🟠 2025 | FLAC collection manager and validator |
| [<small>wally-crunk/</small>​**splatflac**](https://github.com/wally-crunk/splatflac) | - | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | 🐧 | 🔴 Python | 1 | 🟢 2026 | Splits FLAC files using CUE sheets |
| [<small>x1ppy/</small>​**gazelle-origin**](https://github.com/x1ppy/gazelle-origin) | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 43 | 🔴 2023 | Fetches and saves torrent origin metadata |
| [<small>wally-crunk/</small>​**oatgrass**](https://github.com/wally-crunk/oatgrass) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 0 | 🟢 2026 | Cross-upload candidate finder for RED and OPS |
| [<small>passelecasque/</small>​**propolis**](https://gitlab.com/passelecasque/propolis) | - | - | - | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | 🪟 🍎 🐧 | 🟢 Go | 13 | 🔴 2023 | FLAC release checker/validator |
| [<small>aruhier/</small>​**gazelle-uploader**](https://github.com/aruhier/gazelle-uploader) | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 11 | 🔴 2017 | Targeted Apollo/PTH (defunct trackers) |
| [<small>EwolBash/</small>​**spsox**](https://github.com/EwolBash/spsox) | - | - | - | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | 🍎 🐧 | 🔴 Shell | 23 | 🟠 2025 | Parallel spectrogram generator using SoX |
| [<small>_mclovin/</small>​**specdiff**](https://gitlab.com/_mclovin/specdiff) | - | - | - | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Shell | 8 | 🔴 2021 | Spectrogram diffs between two FLACs |
| [<small>passelecasque/</small>​**nectar**](https://gitlab.com/passelecasque/nectar) | - | - | - | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | 🐧 | 🔴 Python | 5 | 🔴 2018 | Detects lossy transcodes masquerading as lossless |


---

#### API

API indicates the tool authenticates via API keys. Tools without API support typically submit login credentials programmatically using browser automation libraries.

#### CI/CD and Tests

CI/CD and tests provide confidence that the project builds and behaves correctly.

- Without CI, broken builds and regressions may go undetected until users report them.
- Without tests, there is no way to verify that changes preserve existing behavior.

#### Platforms
- 🪟 Windows
- 🍎 macOS
- 🐧 Linux
- 🐳 Docker

#### Language

##### 🟢 Rust, Go

- **Install and run**: Download and execute
- **Distribution**: Standalone binaries with no runtime dependencies
- **Type safety**: Type errors, missing fields, and null references are caught at compile time
- **Speed**: Fast. Compiled directly to machine code

##### 🟠 JavaScript, Ruby

- **Install and run**: Consistent package management with reliable dependency resolution
- **Distribution**: Runtime required
- **Type safety**: Lack of safety causes unexpected errors
- **Speed**: Moderate. Node.js optimizes code while running

##### 🔴 Python, Shell

- **Install and run**: Confusing package management with tooling inconsistent between projects
- **Distribution**: Interpreter required with fragmented virtual environment tooling
- **Type safety**: Lack of safety causes unexpected errors
- **Speed**: Slow. Runs as-is without optimization
