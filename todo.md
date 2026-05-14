# Parking Lot

## Link Digest

```text
┌─────────────────────────────┐
│ Config                      │
│ - Bluesky handle/app pass   │
│ - follow filters            │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Identity + Follow Sync      │
│ - Bluesky follows           │
│ - lists / pinned accounts   │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Post Ingest                 │
│ - poll author feeds         │
│ - optional Jetstream        │
│ - backoff / rate limits     │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Link Extractor              │
│ - extract URLs              │
│ - resolve redirects         │
│ - strip tracking params     │
│ - canonicalize              │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ SQLite Store                │
│ - actors                    │
│ - posts                     │
│ - links                     │
│ - shares                    │
│ - daily digests             │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ Ranker                      │
│ - unique sharers            │
│ - recency                   │
│ - domain weights            │
│ - muted keywords/domains    │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ UI / Output                 │
│ - local web app             │
│ - Netscape bookmark file    │
│ - Markdown digest           │
│ - RSS feed                  │
└─────────────────────────────┘
```
