# Output Naming

By default caesura names output folders using the pattern:

```
Artist - Album (Edition Title) [Year] [Media Format]
```

For example: `Mock Artist - Test Album (Deluxe Edition) [2020] [CD FLAC]`

## Static override (`--name`)

Pass a literal string to `--name` to replace the prefix (everything before the `[Media Format]` suffix):

```bash
caesura transcode 142659 --name "My Custom Folder"
```

This produces folders like:
- `My Custom Folder [CD FLAC]`
- `My Custom Folder [CD 320]`
- `My Custom Folder [CD SPECTROGRAMS]`

The value is sanitized for filesystem safety (restricted characters are removed or replaced).

The format suffix (`[WEB FLAC]`, `[CD SPECTROGRAMS]`, etc.) is always appended automatically.

When `--name-template` is set, `--name` is available as `{{ name }}` in the template instead of acting as a prefix override.

## Template mode (`--name-template`)

For full control over folder naming using metadata, use `--name-template` with the `--experimental-name-template` flag:

```bash
caesura transcode 142659 --experimental-name-template \
  --name-template "{{ artist }} - {{ album }} [{{ year }}] [{{ media }} {{ format }}]"
```

Unlike `--name`, the format suffix is **not** appended automatically. The template has full control over the output, including whether to include the format suffix at all.

Templates use [minijinja](https://docs.rs/minijinja) (Jinja2-compatible) syntax.

The default-equivalent template is:

```jinja
{{ artist }} - {{ album }}{% if edition_title %} ({{ edition_title }}){% endif %} [{{ year }}] [{{ media }} {{ format }}]
```

### Available variables

| Variable           | Type            | Description                          | Example                          |
|--------------------|-----------------|--------------------------------------|----------------------------------|
| `artist`           | String          | Artist display name from credits†    | `Mock Artist`, `Various Artists` |
| `album`            | String          | Album title                          | `Test Album`                     |
| `edition_title`    | String or None  | Edition title                        | `Deluxe Edition`                 |
| `year`             | Integer         | Edition year if set, else original   | `2020`                           |
| `original_year`    | Integer or None | Original release year                | `2001`                           |
| `edition_year`     | Integer or None | Edition year                         | `2020`                           |
| `media`            | String          | Media type                           | `CD`, `WEB`, `Vinyl`             |
| `record_label`     | String or None  | Edition record label                 | `Mock Records`                   |
| `catalogue_number` | String or None  | Edition catalogue number             | `0123456789`                     |
| `artists`          | List            | All main artists                     | `["Artist One", "Artist Two"]`   |
| `composers`        | List            | Composers (classical works)          | `["Mock Composer"]`              |
| `conductor`        | List            | Conductors (classical works)         | `["Mock Conductor"]`             |
| `dj`               | List            | DJs                                  | `["Mock DJ"]`                    |
| `producer`         | List            | Producers                            | `["Mock Producer"]`              |
| `remixed_by`       | List            | Remix artists                        | `["Mock Remixer"]`               |
| `with`             | List            | Featured artists                     | `["Guest One", "Guest Two"]`     |
| `arranger`         | List            | Arrangers (OPS only)                 | `["Mock Arranger"]`              |
| `format`           | String or None  | Target format                        | `FLAC`, `320`, `V0`              |
| `spectrogram`      | Boolean         | Whether this is a spectrogram output | `true`, `false`                  |

† Artist derivation:
- 1-2 main artists: joined with `&`
- 3+ main artists: falls back to single DJ or composer, else `Various Artists`
- No main artists: falls back to guest artists, else `Unknown Artist`

This can be replicated as a template:

```jinja
{% if artists|length > 0 and artists|length <= 2 -%}
  {{ artists|and_join }}
{%- elif dj|length == 1 -%}
  {{ dj|first }}
{%- elif composers|length == 1 -%}
  {{ composers|first }}
{%- elif artists|length == 0 and with|length > 0 and with|length <= 2 -%}
  {{ with|and_join }}
{%- elif artists|length == 0 and with|length == 0 -%}
  Unknown Artist
{%- else -%}
  Various Artists
{%- endif %} - {{ album }} [{{ year }}] [{{ media }} {{ format }}]
```

### Conditionals

`None` is falsy in Jinja, so `{% if edition_title %}` naturally skips the block when there is no edition title:

```jinja
{{ artist }} - {{ album }}{% if edition_title %} ({{ edition_title }}){% endif %} [{{ year }}] [{{ media }} {{ format }}]
```

### Built-in filters

- `{{ album|lower }}` - lowercase
- `{{ album|upper }}` - uppercase
- `{{ album|title }}` - title case
- `{{ album|trim }}` - strip whitespace
- `{{ album|truncate(50) }}` - limit length
- `{{ album|replace("foo", "bar") }}` - string substitution
- `{{ album|default("Unknown") }}` - fallback value

### Custom filters

- `{{ items|limit(3) }}` - take at most N items from a list
- `{{ items|and_join }}` - join with commas and `&` before the last item (e.g. `Alice, Bob & Carol`)
- `{{ items|limit(3)|and_join }}` - combine both to cap and join (e.g. `Alice, Bob & Carol`)

### Precedence

1. `--name-template` + `--experimental-name-template` (full template, no automatic suffix)
2. `--name` (static prefix + automatic suffix)
3. Default `Artist - Album (Edition) [Year]` prefix + automatic suffix

### Restricted characters

Both `--name` and `--name-template` are validated at startup. Values containing any of these characters are rejected:

**Removed** (stripped from the rendered output):
- `: < > " ? *` and `.` (invalid or problematic on Windows/macOS filesystems)
- Control characters such as newlines, tabs, and carriage returns (U+0000-U+001F, U+007F)
- Unicode invisibles: non-breaking space (U+00A0), zero-width space (U+200B), BOM/ZWNBSP (U+FEFF), directional marks and overrides (U+200E-U+202E)

**Replaced with `-`**:
- `/` `\` `|` (path separators)
- En dash (U+2013) and em dash (U+2014), normalized to ASCII hyphen

Metadata pulled from the API (artist, album, etc.) can contain any of these characters. The final rendered name is always sanitized before use, so restricted characters in metadata are handled automatically.

### Validation

- If `--name` contains template syntax (`{{`, `}}`, `{%`, `%}`) caesura will report a validation error suggesting you use `--name-template` instead.
- If `--name-template` is set without `--experimental-name-template`, caesura will report a validation error.
- If `--experimental-name-template` is set, the template is rendered with dummy metadata at startup to catch syntax errors early.

### Examples

```bash
# Static override (suffix appended automatically)
caesura transcode 142659 --name "Mock Artist - Some Custom Title"

# Full template control (you supply the suffix)
caesura transcode 142659 --experimental-name-template \
  --name-template "{{ artist }} - {{ album }} [{{ year }}] [{{ media }} {{ format }}]"

# Template with conditional edition title
caesura transcode 142659 --experimental-name-template \
  --name-template "{{ artist }} - {{ album }}{% if edition_title %} ({{ edition_title }}){% endif %} [{{ year }}] [{{ media }} {{ format }}]"

# Minimal template without format suffix
caesura transcode 142659 --experimental-name-template \
  --name-template "{{ album }}"
```
