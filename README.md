# AT-Protocol Tools

A set of tools for working with ATmosphere/AT Protocol powered apps, all around a unified,
clap + owo-colors, posix-compliant CLI.

## Concept

In a `config.toml`, set your handle or DID. If you set your handle, the tool will resolve it to a DID and use that for all operations. See `config.example.toml` for the full shape.

By default the CLI reads `~/.config/atproto-tools/config.toml`. You can pass another path with
`--config`.

### Project Structure

Simple CLI + shared Core crate structure, with separate crates for each app.

```sh
crates
  ├── cli       # Binary entrypoint
  ├── core      # Shared code (Client & Config Management)
  ├── margin
  ├── tangled
  ├── semble
  └── leaflet
```

### Margin (at.margin.* NSID namespace)

Download highlights and notes, and construct a local graph of activity, as well, as build
a markdown file of all highlights and notes (for a specific page).

### Semble (network.cosmik.* NSID namespace)

### Leaflet (site.standard.\* + pub.leaflet.* NSID namespaces)

### Tangled

Just the `sh.tangled.string` lexicon
