# AT-Protocol Tools

A set of tools for working with ATmosphere/AT Protocol powered apps, all around a unified,
clap + owo-colors, posix-compliant CLI.

## Config

In a `config.toml`, set your handle or DID. If you set your handle, the tool will resolve it to a DID and use that for all operations. See `config.example.toml` for the full shape.

By default the CLI reads `~/.config/atproto-tools/config.toml`. You can pass another path with
`--config`.

## Commands

`atp [--config PATH] COMMAND [ARGS]`

Run AT Protocol inspection, lexicon, and app-specific commands.

- `--config PATH` — Read and write configuration at `PATH`.

`atp info [--actor HANDLE_OR_DID] [--json]`

Fetch profile metadata from the public Bluesky API, resolve the actor's DID document, and describe their repository.

- `--actor HANDLE_OR_DID` — Inspect this actor instead of `identity.identifier` from config.
- `--json` — Print the complete response as formatted JSON.

`atp config set FIELD VALUE`

Set a supported config field. Creates the config file and parent directory if needed.

`atp lexicons sync TOOL --commit COMMIT [--repo REPO] [--source-path PATH] [--dest DIR] [--file FILE]...`

Pull selected Lexicon JSON files from a git repository at an explicit commit hash. `TOOL` selects the default source repo, source path, local destination, and file set.

- `--commit COMMIT` — Fetch lexicons from this commit.
- `--repo REPO` — Override the source repository.
- `--source-path PATH` — Override the directory inside the source repository containing lexicons.
- `--dest DIR` — Override the local destination directory.
- `--file FILE` — Sync this lexicon file. Repeat to replace tool defaults.

`atp lexicons generate TOOL [--input DIR] [--output FILE]`

Read local Lexicon JSON and write serde-compatible Rust structs for a specific tool crate.

`atp margin export [--actor HANDLE_OR_DID] [--source URL] [--output-dir DIR]`

Fetch `at.margin.note` records from the actor's resolved PDS, group them by `target.source`, and write Obsidian/GFM-compatible Markdown documents with TOML frontmatter. Without `--source`, writes one slugified file per source.

`atp bsky follows [--actor HANDLE_OR_DID] [--limit N] [--sort FIELD] [--asc | --desc] [--sa FIELD | --sd FIELD] [--refresh] [--json]`

Fetch followed accounts and each account's latest original post. Results include handle, DID, profile URL, latest post date and URL. Reports are cached under `~/.cache/atproto-tools/bsky-follows`.

- `--actor HANDLE_OR_DID` — Inspect this actor instead of `identity.identifier` from config.
- `--limit N` — Inspect only the first `N` follows.
- `--sort FIELD` — Sort by `FIELD`: `handle`, `did`, `profile-url`, `last-post-at`, `last-post-rkey`, `last-post-url` (camelCase aliases accepted). Default: `last-post-at`.
- `--asc` / `--desc` — Sort direction (default: ascending).
- `--sa FIELD` / `--sd FIELD` — Sort by `FIELD` ascending/descending.
- `--refresh` — Ignore cache and fetch fresh data.
- `--json` — Print the complete cached report as formatted JSON.

### Project Structure

Simple CLI + shared Core crate structure, with separate crates for each app.

```sh
crates
  ├── cli       # Binary entrypoint
  ├── core      # Shared code (Client & Config Management)
  ├── margin
  ├── tngl
  ├── semble
  ├── bsky
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

### BlueSky (app.bsky.* NSID namespace)

Fetches and analyzes data from the public Bluesky API (`public.api.bsky.app`).

**`atp bsky follows`** resolves the actor's profile, then paginates through all follows
via `app.bsky.graph.getFollows`.
For each follow it fetches the latest original post (non-repost, authored by that account)
via `app.bsky.feed.getAuthorFeed`, up to 8 concurrent requests with a 50 ms stagger.

Results include handle, DID, profile URL, last post timestamp, and last post URL.

Reports are cached as JSON under `~/.cache/atproto-tools/bsky-follows/`, keyed with a
SHA-256 hash of the actor's DID, handle, follows count.
