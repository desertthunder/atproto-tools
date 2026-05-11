CREATE TABLE actor_last_post (
  did TEXT PRIMARY KEY,
  last_post_at TEXT,
  last_post_uri TEXT,
  source TEXT NOT NULL CHECK (
    source IN ('author_feed', 'jetstream', 'repair', 'unknown')
  ),
  confidence TEXT NOT NULL CHECK (
    confidence IN ('fresh', 'stale', 'unknown', 'empty')
  ),
  checked_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_actor_last_post_checked_at
ON actor_last_post(checked_at);

CREATE INDEX idx_actor_last_post_last_post_at
ON actor_last_post(last_post_at);

CREATE TABLE backfill_jobs (
  did TEXT PRIMARY KEY,
  status TEXT NOT NULL CHECK (
    status IN ('queued', 'running', 'done', 'failed')
  ),
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  queued_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_backfill_jobs_status
ON backfill_jobs(status, queued_at);

CREATE TABLE jetstream_state (
  id TEXT PRIMARY KEY,
  cursor_us INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);
