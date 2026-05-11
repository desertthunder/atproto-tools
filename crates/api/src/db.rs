use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use std::path::PathBuf;
use tokio_rusqlite::Connection;
use tokio_rusqlite::rusqlite::{Error as SqliteError, params};

const MIGRATIONS: &[Migration] =
    &[Migration { version: 1, name: "initial", sql: include_str!("../migrations/0001_initial.sql") }];

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug, Clone)]
pub struct DbActorLastPost {
    pub did: String,
    pub last_post_at: Option<String>,
    pub last_post_uri: Option<String>,
    pub source: String,
    pub confidence: String,
    pub checked_at: String,
}

pub async fn open_database(path: PathBuf) -> Result<Connection> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create database directory {}", parent.display()))?;
    }

    let db = Connection::open(&path)
        .await
        .with_context(|| format!("failed to open database {}", path.display()))?;
    migrate_database(&db).await?;
    Ok(db)
}

async fn migrate_database(db: &Connection) -> Result<()> {
    db.call(|conn| {
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            ",
        )?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
              version INTEGER PRIMARY KEY,
              name TEXT NOT NULL,
              applied_at TEXT NOT NULL
            );
            ",
        )?;

        for migration in MIGRATIONS {
            let applied = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                params![migration.version],
                |row| row.get::<_, bool>(0),
            )?;

            if applied {
                continue;
            }

            let tx = conn.transaction()?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                params![migration.version, migration.name, now_string()],
            )?;
            tx.commit()?;
        }

        Ok::<(), SqliteError>(())
    })
    .await
    .context("failed to migrate database")
}

pub async fn ready(db: &Connection) -> Result<()> {
    db.call(|conn| conn.query_row("SELECT 1", [], |_| Ok(())))
        .await
        .context("database readiness check failed")
}

pub async fn load_actor_last_posts(db: &Connection, actors: Vec<String>) -> Result<Vec<DbActorLastPost>> {
    db.call(move |conn| {
        let mut rows = Vec::new();
        let mut statement = conn.prepare(
            "
            SELECT did, last_post_at, last_post_uri, source, confidence, checked_at
            FROM actor_last_post
            WHERE did = ?1
            ",
        )?;

        for actor in actors {
            let result = statement.query_row(params![actor], |row| {
                Ok(DbActorLastPost {
                    did: row.get(0)?,
                    last_post_at: row.get(1)?,
                    last_post_uri: row.get(2)?,
                    source: row.get(3)?,
                    confidence: row.get(4)?,
                    checked_at: row.get(5)?,
                })
            });

            match result {
                Ok(row) => rows.push(row),
                Err(SqliteError::QueryReturnedNoRows) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(rows)
    })
    .await
    .context("failed to load actor last posts")
}

pub async fn persist_backfill_job(db: &Connection, did: &str) -> Result<()> {
    let did = did.to_string();
    let now = now_string();
    db.call(move |conn| {
        conn.execute(
            "
            INSERT INTO backfill_jobs (did, status, attempts, queued_at, updated_at)
            VALUES (?1, 'queued', 0, ?2, ?2)
            ON CONFLICT(did) DO UPDATE SET
              status = CASE
                WHEN backfill_jobs.status = 'running' THEN backfill_jobs.status
                ELSE 'queued'
              END,
              queued_at = excluded.queued_at,
              updated_at = excluded.updated_at
            ",
            params![did, now],
        )?;
        Ok::<(), SqliteError>(())
    })
    .await
    .context("failed to persist backfill job")
}

pub async fn mark_job_running(db: &Connection, did: &str) -> Result<()> {
    update_job_status(db, did, "running", None, false).await
}

pub async fn mark_job_done(db: &Connection, did: &str) -> Result<()> {
    update_job_status(db, did, "done", None, false).await
}

pub async fn mark_job_failed(db: &Connection, did: &str, error: &str) -> Result<()> {
    update_job_status(db, did, "failed", Some(error), true).await
}

async fn update_job_status(
    db: &Connection, did: &str, status: &str, last_error: Option<&str>, increment_attempts: bool,
) -> Result<()> {
    let did = did.to_string();
    let status = status.to_string();
    let last_error = last_error.map(str::to_string);
    let now = now_string();
    db.call(move |conn| {
        conn.execute(
            "
            UPDATE backfill_jobs
            SET status = ?2,
                attempts = attempts + ?3,
                last_error = ?4,
                updated_at = ?5
            WHERE did = ?1
            ",
            params![did, status, i64::from(increment_attempts), last_error, now],
        )?;
        Ok::<(), SqliteError>(())
    })
    .await
    .context("failed to update backfill job")
}

pub async fn upsert_actor_last_post(
    db: &Connection, did: &str, last_post_at: Option<&str>, last_post_uri: Option<&str>, source: &str, confidence: &str,
) -> Result<()> {
    let did = did.to_string();
    let last_post_at = last_post_at.map(str::to_string);
    let last_post_uri = last_post_uri.map(str::to_string);
    let source = source.to_string();
    let confidence = confidence.to_string();
    let now = now_string();

    db.call(move |conn| {
        conn.execute(
            "
            INSERT INTO actor_last_post (
              did, last_post_at, last_post_uri, source, confidence, checked_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ON CONFLICT(did) DO UPDATE SET
              last_post_at = CASE
                WHEN excluded.last_post_at IS NULL THEN actor_last_post.last_post_at
                WHEN actor_last_post.last_post_at IS NULL THEN excluded.last_post_at
                WHEN excluded.last_post_at > actor_last_post.last_post_at THEN excluded.last_post_at
                ELSE actor_last_post.last_post_at
              END,
              last_post_uri = CASE
                WHEN excluded.last_post_at IS NULL THEN actor_last_post.last_post_uri
                WHEN actor_last_post.last_post_at IS NULL THEN excluded.last_post_uri
                WHEN excluded.last_post_at > actor_last_post.last_post_at THEN excluded.last_post_uri
                ELSE actor_last_post.last_post_uri
              END,
              source = CASE
                WHEN excluded.last_post_at IS NULL AND actor_last_post.last_post_at IS NOT NULL THEN actor_last_post.source
                WHEN actor_last_post.last_post_at IS NULL THEN excluded.source
                WHEN excluded.last_post_at > actor_last_post.last_post_at THEN excluded.source
                ELSE actor_last_post.source
              END,
              confidence = excluded.confidence,
              checked_at = excluded.checked_at,
              updated_at = excluded.updated_at
            ",
            params![did, last_post_at, last_post_uri, source, confidence, now],
        )?;
        Ok::<(), SqliteError>(())
    })
    .await
    .context("failed to upsert actor last post")
}

pub async fn load_jetstream_cursor(db: &Connection) -> Result<Option<i64>> {
    db.call(|conn| {
        let result = conn.query_row(
            "SELECT cursor_us FROM jetstream_state WHERE id = 'default'",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(cursor) => Ok(Some(cursor)),
            Err(SqliteError::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    })
    .await
    .context("failed to load Jetstream cursor")
}

pub async fn save_jetstream_cursor(db: &Connection, cursor: i64) -> Result<()> {
    let now = now_string();
    db.call(move |conn| {
        conn.execute(
            "
            INSERT INTO jetstream_state (id, cursor_us, updated_at)
            VALUES ('default', ?1, ?2)
            ON CONFLICT(id) DO UPDATE SET
              cursor_us = excluded.cursor_us,
              updated_at = excluded.updated_at
            ",
            params![cursor, now],
        )?;
        Ok::<(), SqliteError>(())
    })
    .await
    .context("failed to save Jetstream cursor")
}

pub fn now_string() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
