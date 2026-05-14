use super::db;
use anyhow::Context;
use atp_tools_bsky::fetch_actor_top_level_last_post;
use atp_tools_core::AtprotoClient;
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::{interval, sleep, timeout};
use tokio_rusqlite::Connection;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize)]
struct JetstreamEvent {
    did: String,
    time_us: i64,
    kind: String,
    commit: Option<JetstreamCommit>,
}

#[derive(Debug, Deserialize)]
struct JetstreamCommit {
    operation: String,
    collection: String,
    rkey: String,
    record: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JetstreamPostRecord {
    #[serde(rename = "$type")]
    r#type: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    reply: Option<serde_json::Value>,
}

#[derive(Debug, Default)]
struct WorkerStats {
    connected: AtomicBool,
    reconnects: AtomicU64,
    events: AtomicU64,
    indexed: AtomicU64,
    parse_errors: AtomicU64,
    db_errors: AtomicU64,
    last_cursor_us: AtomicI64,
}

pub async fn run_worker(
    db: Arc<Connection>, client: AtprotoClient, jetstream_endpoint: String, seed_actor: String, heartbeat_seconds: u64,
    resume: bool,
) -> anyhow::Result<()> {
    tokio::spawn(seed_actor_task(db.clone(), client.clone(), seed_actor));
    let stats = Arc::new(WorkerStats::default());
    tokio::spawn(heartbeat_task(stats.clone(), heartbeat_seconds));

    let mut cursor = if resume { db::load_jetstream_cursor(&db).await? } else { None };
    if let Some(cursor) = cursor {
        stats.last_cursor_us.store(cursor, Ordering::Relaxed);
    }

    loop {
        let mut url = jetstream_endpoint.to_string();
        let separator = if jetstream_endpoint.contains('?') { '&' } else { '?' };
        url.push(separator);
        url.push_str("wantedCollections=app.bsky.feed.post");

        if let Some(cursor) = cursor {
            url.push('&');
            url.push_str("cursor=");
            url.push_str(&cursor.to_string());
        };

        eprintln!("worker connecting endpoint={url}");

        match connect_async(&url).await {
            Ok((stream, _response)) => {
                stats.connected.store(true, Ordering::Relaxed);
                eprintln!("worker connected endpoint={jetstream_endpoint}");
                let (_write, mut read) = stream.split();

                while let Some(message) = read.next().await {
                    match message {
                        Ok(message) => match message {
                            Message::Text(text) => {
                                if let Some(next_cursor) = handle_jetstream_text(&db, text.as_str(), &stats).await {
                                    cursor = Some(next_cursor);
                                }
                            }
                            Message::Binary(bytes) => match std::str::from_utf8(&bytes) {
                                Ok(text) => {
                                    if let Some(next_cursor) = handle_jetstream_text(&db, text, &stats).await {
                                        cursor = Some(next_cursor);
                                    }
                                }
                                Err(error) => {
                                    stats.parse_errors.fetch_add(1, Ordering::Relaxed);
                                    tracing::warn!(?error, "received non-UTF-8 Jetstream binary message");
                                }
                            },
                            Message::Close(frame) => {
                                eprintln!("worker websocket_closed frame={frame:?}");
                            }
                            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                        },
                        Err(error) => {
                            tracing::warn!(?error, "Jetstream read failed");
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                tracing::warn!(?error, "Jetstream connection failed")
            }
        }

        stats.connected.store(false, Ordering::Relaxed);
        stats.reconnects.fetch_add(1, Ordering::Relaxed);
        eprintln!("worker disconnected reconnect_in_seconds=5");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn heartbeat_task(stats: Arc<WorkerStats>, heartbeat_seconds: u64) {
    let heartbeat_seconds = heartbeat_seconds.max(1);
    let mut interval = interval(Duration::from_secs(heartbeat_seconds));

    loop {
        interval.tick().await;
        eprintln!(
            "worker heartbeat connected={} events={} indexed={} reconnects={} parse_errors={} db_errors={} cursor_us={}",
            stats.connected.load(Ordering::Relaxed),
            stats.events.load(Ordering::Relaxed),
            stats.indexed.load(Ordering::Relaxed),
            stats.reconnects.load(Ordering::Relaxed),
            stats.parse_errors.load(Ordering::Relaxed),
            stats.db_errors.load(Ordering::Relaxed),
            stats.last_cursor_us.load(Ordering::Relaxed),
        );
    }
}

async fn seed_actor_task(db: Arc<Connection>, client: AtprotoClient, seed_actor: String) {
    match timeout(
        Duration::from_secs(10),
        seed_actor_from_author_feed(&db, &client, &seed_actor),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(error)) => tracing::warn!(%seed_actor, ?error, "failed to seed actor from author feed"),
        Err(_) => tracing::warn!(%seed_actor, "timed out seeding actor from author feed"),
    }
}

async fn seed_actor_from_author_feed(db: &Connection, client: &AtprotoClient, seed_actor: &str) -> anyhow::Result<()> {
    let did = client
        .resolve_actor_did(seed_actor)
        .await
        .with_context(|| format!("failed to resolve seed actor {seed_actor}"))?;

    match fetch_actor_top_level_last_post(client, &did).await? {
        Some(post) => {
            db::upsert_actor_last_post(
                db,
                &did,
                Some(&post.created_at),
                Some(&post.uri),
                "author_feed",
                "fresh",
            )
            .await
        }
        None => db::upsert_actor_last_post(db, &did, None, None, "author_feed", "empty").await,
    }
}

async fn handle_jetstream_text(db: &Connection, text: &str, stats: &WorkerStats) -> Option<i64> {
    let event = match serde_json::from_str::<JetstreamEvent>(text) {
        Ok(event) => event,
        Err(error) => {
            stats.parse_errors.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(?error, "failed to parse Jetstream event");
            return None;
        }
    };
    stats.events.fetch_add(1, Ordering::Relaxed);
    stats.last_cursor_us.store(event.time_us, Ordering::Relaxed);

    if let Err(error) = db::save_jetstream_cursor(db, event.time_us).await {
        stats.db_errors.fetch_add(1, Ordering::Relaxed);
        tracing::warn!(cursor = event.time_us, ?error, "failed to save Jetstream cursor");
    }

    if event.kind != "commit" {
        return Some(event.time_us);
    }

    if let Some((created_at, uri)) = jetstream_top_level_post(&event) {
        if let Err(error) =
            db::upsert_actor_last_post(db, &event.did, Some(&created_at), Some(&uri), "jetstream", "fresh").await
        {
            stats.db_errors.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(did = %event.did, ?error, "failed to index Jetstream post");
        } else {
            stats.indexed.fetch_add(1, Ordering::Relaxed);
        }
    }

    Some(event.time_us)
}

fn jetstream_top_level_post(event: &JetstreamEvent) -> Option<(String, String)> {
    let commit = event.commit.as_ref()?;
    if commit.operation != "create" || commit.collection != "app.bsky.feed.post" {
        return None;
    }

    let record_value = commit.record.as_ref()?;
    let record = serde_json::from_value::<JetstreamPostRecord>(record_value.clone()).ok()?;
    if record.reply.is_some() {
        return None;
    }

    if let Some(record_type) = record.r#type.as_deref() {
        if record_type != "app.bsky.feed.post" {
            return None;
        }
    }

    let uri = format!("at://{}/{}/{}", event.did, commit.collection, commit.rkey);
    Some((record.created_at, uri))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_top_level_post_from_jetstream_event() {
        let event = serde_json::from_value::<JetstreamEvent>(json!({
            "did": "did:plc:abc",
            "time_us": 1725911162329308i64,
            "kind": "commit",
            "commit": {
                "operation": "create",
                "collection": "app.bsky.feed.post",
                "rkey": "3abc",
                "record": {
                    "$type": "app.bsky.feed.post",
                    "text": "hello",
                    "createdAt": "2026-05-11T12:00:00.000Z"
                }
            }
        }))
        .expect("event");

        assert_eq!(
            jetstream_top_level_post(&event),
            Some((
                "2026-05-11T12:00:00.000Z".to_string(),
                "at://did:plc:abc/app.bsky.feed.post/3abc".to_string(),
            ))
        );
    }

    #[test]
    fn skips_replies_from_jetstream_event() {
        let event = serde_json::from_value::<JetstreamEvent>(json!({
            "did": "did:plc:abc",
            "time_us": 1725911162329308i64,
            "kind": "commit",
            "commit": {
                "operation": "create",
                "collection": "app.bsky.feed.post",
                "rkey": "3abc",
                "record": {
                    "$type": "app.bsky.feed.post",
                    "text": "hello",
                    "createdAt": "2026-05-11T12:00:00.000Z",
                    "reply": {
                        "root": {"uri": "at://did:plc:abc/app.bsky.feed.post/3root", "cid": "bafy"},
                        "parent": {"uri": "at://did:plc:abc/app.bsky.feed.post/3root", "cid": "bafy"}
                    }
                }
            }
        }))
        .expect("event");

        assert_eq!(jetstream_top_level_post(&event), None);
    }
}
