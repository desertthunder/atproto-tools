# AT Protocol Tools

A set of tools for working with ATmosphere/AT Protocol powered apps, all around a unified,
clap + owo-colors, posix-compliant CLI, and some browser stuff.

## Config

In a `config.toml`, set your handle or DID. If you set your handle, the tool will resolve it to a DID and use that for all operations. See `config.example.toml` for the full shape.

By default the CLI reads `~/.config/atproto-tools/config.toml`. You can pass another path with
`--config`.

## Commands

See `docs/*.txt` for manuals.

### Project Structure

Simple CLI + shared Core crate structure, plus a static web graph app.

```sh
apps
  └── web       # SvelteKit social graph visualizer

crates
  ├── cli       # Binary entrypoint
  ├── core      # Shared code (Client & Config Management)
  ├── margin
  ├── tngl
  ├── semble
  ├── bsky
  └── leaflet
```

### Margin (at.margin.\* NSID namespace)

Download highlights and notes, and construct a local graph of activity, as well, as build
a markdown file of all highlights and notes (for a specific page).

### Semble (network.cosmik.\* NSID namespace)

### Leaflet (site.standard.\* + pub.leaflet.\* NSID namespaces)

### Tangled

Generated models for the `sh.tangled.string`, `sh.tangled.repo`, and `sh.tangled.issue` lexicons to
produce markdown files for strings (TODO).

For repos and issues, we want to generate task lists (TODO).

### BlueSky (app.bsky.\* NSID namespace)

Fetches and analyzes data from the public Bluesky API (`public.api.bsky.app`).

**`atp bsky follows`** resolves the actor's profile, then paginates through all follows
via `app.bsky.graph.getFollows`.
For each follow it fetches the latest original post (non-repost, authored by that account)
via `app.bsky.feed.getAuthorFeed`, up to 8 concurrent requests with a 50 ms stagger.

Results include handle, DID, profile URL, last post timestamp, and last post URL.

Reports are cached as JSON under `~/.cache/atproto-tools/bsky-follows/`, keyed with a
SHA-256 hash of the actor's DID, handle, follows count.

### BlueSky Social Graph Visualizer

The web app in `apps/web` is a static SvelteKit social graph visualizer
for Bluesky accounts. Enter a handle to map followers, following, mutuals,
relationships between rendered nodes, and optional second-hop expansions while
keeping the graph shareable through normal routes and query params.
