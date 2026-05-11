use super::db;
use atp_tools_bsky::fetch_actor_top_level_last_post;
use atp_tools_core::AtprotoClient;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::mpsc};
use tokio_rusqlite::Connection;

pub const DEFAULT_BIND: &str = "127.0.0.1:3000";

const GET_ACTOR_LAST_POSTS_NSID: &str = "dev.desertthunder.atpTools.getActorLastPosts";
const MAX_ACTORS: usize = 100;
const DEFAULT_STALE_AFTER_SECONDS: i64 = 86_400;
const MIN_STALE_AFTER_SECONDS: i64 = 60;
const MAX_STALE_AFTER_SECONDS: i64 = 604_800;
const BACKFILL_QUEUE_SIZE: usize = 1024;

#[derive(Clone)]
struct AppState {
    db: Arc<Connection>,
    backfill_tx: mpsc::Sender<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetActorLastPostsRequest {
    actors: Vec<String>,
    stale_after_seconds: Option<i64>,
    enqueue_backfill: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetActorLastPostsResponse {
    items: Vec<ActorLastPostResponse>,
    missing: Vec<String>,
    queued: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActorLastPostResponse {
    did: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_post_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_post_uri: Option<String>,
    source: String,
    confidence: String,
    checked_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceStatus {
    ok: bool,
    service: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error: &'static str,
    message: String,
}

struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self.0, "request failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody { error: "InternalServerError", message: "internal server error".to_string() }),
        )
            .into_response()
    }
}

pub async fn run_server(
    bind: SocketAddr, db: Arc<Connection>, client: AtprotoClient, seed_actor: String,
) -> anyhow::Result<()> {
    let (backfill_tx, backfill_rx) = mpsc::channel(BACKFILL_QUEUE_SIZE);
    tokio::spawn(run_backfill_worker(db.clone(), client.clone(), backfill_rx));

    match client.resolve_actor_did(&seed_actor).await {
        Ok(did) => {
            if enqueue_actor(&db, &backfill_tx, &did).await {
                tracing::info!(%seed_actor, %did, "queued seed actor backfill");
            }
        }
        Err(error) => tracing::warn!(%seed_actor, ?error, "failed to resolve seed actor"),
    }

    let state = AppState { db, backfill_tx };
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            &format!("/xrpc/{GET_ACTOR_LAST_POSTS_NSID}"),
            post(get_actor_last_posts),
        )
        .with_state(state);

    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "serving API");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn root() -> Json<ServiceStatus> {
    Json(ServiceStatus { ok: true, service: "atp-tools-api" })
}

async fn healthz() -> Json<ServiceStatus> {
    Json(ServiceStatus { ok: true, service: "atp-tools-api" })
}

async fn readyz(State(state): State<AppState>) -> Result<Json<ServiceStatus>, ApiError> {
    db::ready(&state.db).await?;
    Ok(Json(ServiceStatus { ok: true, service: "atp-tools-api" }))
}

async fn get_actor_last_posts(
    State(state): State<AppState>, Json(request): Json<GetActorLastPostsRequest>,
) -> Result<Response, ApiError> {
    let stale_after_seconds = request.stale_after_seconds.unwrap_or(DEFAULT_STALE_AFTER_SECONDS);
    if !(MIN_STALE_AFTER_SECONDS..=MAX_STALE_AFTER_SECONDS).contains(&stale_after_seconds) {
        return Ok(bad_request(format!(
            "staleAfterSeconds must be between {MIN_STALE_AFTER_SECONDS} and {MAX_STALE_AFTER_SECONDS}"
        )));
    }

    let actors = match validate_actors(request.actors) {
        Ok(actors) => actors,
        Err(message) => return Ok(bad_request(message)),
    };
    let enqueue_backfill = request.enqueue_backfill.unwrap_or(true);
    let rows = db::load_actor_last_posts(&state.db, actors.clone()).await?;
    let now = Utc::now();

    let mut items = Vec::new();
    let mut missing = Vec::new();
    let mut queued = Vec::new();

    for actor in actors {
        match rows.iter().find(|row| row.did == actor) {
            Some(row) => {
                let confidence = {
                    if row.confidence == "empty" {
                        row.confidence.clone()
                    } else {
                        match DateTime::parse_from_rfc3339(&row.checked_at) {
                            Ok(checked_at) => {
                                let age = now.signed_duration_since(checked_at.with_timezone(&Utc));
                                if age.num_seconds() > stale_after_seconds {
                                    "stale".to_string()
                                } else {
                                    "fresh".to_string()
                                }
                            }
                            Err(_) => "unknown".to_string(),
                        }
                    }
                };

                if confidence == "stale"
                    && enqueue_backfill
                    && enqueue_actor(&state.db, &state.backfill_tx, &actor).await
                {
                    queued.push(actor.clone());
                }
                items.push(ActorLastPostResponse {
                    did: row.did.clone(),
                    last_post_at: row.last_post_at.clone(),
                    last_post_uri: row.last_post_uri.clone(),
                    source: match row.source.as_str() {
                        "author_feed" => "authorFeed",
                        "jetstream" => "jetstream",
                        "repair" => "repair",
                        _ => "unknown",
                    }
                    .to_string(),
                    confidence,
                    checked_at: row.checked_at.clone(),
                });
            }
            None => {
                missing.push(actor.clone());
                if enqueue_backfill && enqueue_actor(&state.db, &state.backfill_tx, &actor).await {
                    queued.push(actor);
                }
            }
        }
    }

    Ok(Json(GetActorLastPostsResponse { items, missing, queued }).into_response())
}

fn bad_request(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody { error: "InvalidRequest", message }),
    )
        .into_response()
}

fn validate_actors(actors: Vec<String>) -> Result<Vec<String>, String> {
    if actors.is_empty() {
        return Err("actors must contain at least one DID".to_string());
    }

    if actors.len() > MAX_ACTORS {
        return Err(format!("actors must contain at most {MAX_ACTORS} DIDs"));
    }

    let mut seen = HashSet::new();
    let mut validated = Vec::with_capacity(actors.len());
    for actor in actors {
        if !(actor.starts_with("did:") && !actor.chars().any(char::is_whitespace)) {
            return Err(format!("actor {actor:?} must be a DID"));
        }

        if seen.insert(actor.clone()) {
            validated.push(actor);
        }
    }

    Ok(validated)
}

async fn enqueue_actor(db: &Connection, backfill_tx: &mpsc::Sender<String>, did: &str) -> bool {
    if let Err(error) = db::persist_backfill_job(db, did).await {
        tracing::warn!(%did, ?error, "failed to persist backfill job");
        return false;
    }

    match backfill_tx.try_send(did.to_string()) {
        Ok(()) => true,
        Err(error) => {
            tracing::warn!(%did, ?error, "failed to enqueue in-process backfill job");
            false
        }
    }
}

async fn run_backfill_worker(db: Arc<Connection>, client: AtprotoClient, mut rx: mpsc::Receiver<String>) {
    while let Some(did) = rx.recv().await {
        if let Err(error) = backfill_actor(&db, &client, &did).await {
            tracing::warn!(%did, ?error, "backfill failed");
        }
    }
}

async fn backfill_actor(db: &Connection, client: &AtprotoClient, did: &str) -> anyhow::Result<()> {
    db::mark_job_running(db, did).await?;

    match fetch_actor_top_level_last_post(client, did).await {
        Ok(Some(post)) => {
            db::upsert_actor_last_post(db, did, Some(&post.created_at), Some(&post.uri), "author_feed", "fresh")
                .await?;
            db::mark_job_done(db, did).await?;
            tracing::debug!(%did, uri = %post.uri, "backfilled actor last post");
            Ok(())
        }
        Ok(None) => {
            db::upsert_actor_last_post(db, did, None, None, "author_feed", "empty").await?;
            db::mark_job_done(db, did).await?;
            tracing::debug!(%did, "backfilled empty actor last post");
            Ok(())
        }

        Err(error) => {
            db::mark_job_failed(db, did, &error.to_string()).await?;
            Err(error.into())
        }
    }
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(?error, "failed to install Ctrl-C handler");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_actor_batch() {
        let actors = validate_actors(vec![
            "did:plc:abc".to_string(),
            "did:plc:abc".to_string(),
            "did:web:example.com".to_string(),
        ])
        .expect("valid actors");

        assert_eq!(actors, ["did:plc:abc", "did:web:example.com"]);
    }

    #[test]
    fn rejects_handles() {
        let error = validate_actors(vec!["desertthunder.dev".to_string()]).expect_err("handle should be rejected");
        assert!(error.contains("must be a DID"));
    }
}
