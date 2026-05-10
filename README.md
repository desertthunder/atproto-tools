# AT-Protocol Tools

A set of tools for working with ATmosphere/AT Protocol powered apps, all around a unified,
clap + owo-colors, posix-compliant CLI.

## Config

In a `config.toml`, set your handle or DID. If you set your handle, the tool will resolve it to a DID and use that for all operations. See `config.example.toml` for the full shape.

By default the CLI reads `~/.config/atproto-tools/config.toml`. You can pass another path with
`--config`.

## Usage

`info` fetches profile metadata from the public Bluesky API, resolves the actor's DID document,
and uses the advertised PDS to describe the actor's repository. By default it prints a compact
key-value summary, with a JSON mode available for the full serde-serialized response.

`config` reads and updates the TOML configuration used by the CLI. It uses the implicit config
path unless `--config` is passed, and it creates the config file and parent directory when setting
fields.

`lexicons sync <tool>` pulls selected Lexicon JSON files from a git repository at an explicit
commit hash. The named tool selects the default source repository, source paths, local destination,
and file set; `--repo`, `--source-path`, `--dest`, and repeated `--file` flags can override those
defaults.

`lexicons generate <tool>` reads local Lexicon JSON and writes serde-compatible Rust structs
for a specific tool crate.

`margin export` fetches `at.margin.note` records from the actor's resolved PDS, groups them by
`target.source`, and writes Obsidian/GFM-compatible Markdown documents with TOML frontmatter.
Pass `--source` to export a single source; otherwise, it writes one slugified file per source.

### Project Structure

Simple CLI + shared Core crate structure, with separate crates for each app.

```sh
crates
  ├── cli       # Binary entrypoint
  ├── core      # Shared code (Client & Config Management)
  ├── margin
  ├── tngl
  ├── semble
  └── leaflet
```

### Margin (at.margin.* NSID namespace)

Download highlights and notes, and construct a local graph of activity, as well, as build
a markdown file of all highlights and notes (for a specific page).

### Semble (network.cosmik.* NSID namespace)

### Leaflet (site.standard.\* + pub.leaflet.* NSID namespaces)

### Tangled

Generated models for the `sh.tangled.string`, `sh.tangled.repo`, and `sh.tangled.issue` lexicons to
produce markdown files for strings (TODO).

For repos and issues, we want to generate task lists (TODO).

### BlueSky
