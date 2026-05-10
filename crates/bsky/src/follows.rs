use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use atp_tools_core::{
    AtprotoClient, ClientError, ParallelConfig, ParallelTaskError, run_parallel_rate_limited_with_progress,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{FeedDefsFeedViewPost, GetAuthorFeedOutput, GetFollowsOutput};

const CACHE_VERSION: u8 = 1;
const FOLLOWS_METHOD: &str = "app.bsky.graph.getFollows";
const AUTHOR_FEED_METHOD: &str = "app.bsky.feed.getAuthorFeed";
const LAST_POST_MAX_PARALLEL: usize = 8;
const LAST_POST_START_DELAY_MS: u64 = 50;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowsReport {
    pub actor: String,
    pub actor_did: String,
    pub cache_key: String,
    pub cache_path: PathBuf,
    pub generated_at_unix: u64,
    pub follows: Vec<FollowLastPost>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowLastPost {
    pub handle: String,
    pub did: String,
    pub profile_url: String,
    pub last_post_at: Option<String>,
    pub last_post_rkey: Option<String>,
    pub last_post_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowsProgress {
    ResolvingActor,
    CheckingCache { path: PathBuf },
    CacheHit { path: PathBuf, count: usize },
    FetchingFollowsPage { page: usize },
    FetchedFollowsPage { page: usize, total: usize },
    FetchingLastPosts { completed: usize, total: usize },
    WritingCache { path: PathBuf },
    WroteCache { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheFile {
    version: u8,
    actor: String,
    actor_did: String,
    cache_key: String,
    generated_at_unix: u64,
    follows: Vec<FollowLastPost>,
}

pub async fn fetch_follows_report(
    client: &AtprotoClient, actor: &str, refresh: bool,
) -> Result<FollowsReport, FollowsReportError> {
    fetch_follows_report_with_progress(client, actor, refresh, |_| {}).await
}

pub async fn fetch_follows_report_with_progress<P>(
    client: &AtprotoClient, actor: &str, refresh: bool, mut progress: P,
) -> Result<FollowsReport, FollowsReportError>
where
    P: FnMut(FollowsProgress) + Send,
{
    progress(FollowsProgress::ResolvingActor);
    let profile = client.get_profile(actor).await?;
    let cache_key = cache_key(
        &profile.did,
        &profile.handle,
        profile.follows_count,
        profile.indexed_at.as_deref(),
    );
    let cache_path = cache_path(&profile.did, &cache_key)?;

    if !refresh {
        progress(FollowsProgress::CheckingCache { path: cache_path.clone() });
        if let Some(report) = read_cache(&cache_path)? {
            progress(FollowsProgress::CacheHit { path: cache_path.clone(), count: report.follows.len() });
            return Ok(report.into_report(cache_path));
        }
    }

    let follows = fetch_all_follows(client, actor, &mut progress).await?;
    let rows = fetch_follows_last_posts(client.clone(), follows, &mut progress).await?;

    let cache_file = CacheFile {
        version: CACHE_VERSION,
        actor: profile.handle,
        actor_did: profile.did,
        cache_key,
        generated_at_unix: now_unix(),
        follows: rows,
    };

    progress(FollowsProgress::WritingCache { path: cache_path.clone() });
    write_cache(&cache_path, &cache_file)?;
    progress(FollowsProgress::WroteCache { path: cache_path.clone() });
    Ok(cache_file.into_report(cache_path))
}

async fn fetch_all_follows<P>(
    client: &AtprotoClient, actor: &str, progress: &mut P,
) -> Result<Vec<super::ActorDefsProfileView>, FollowsReportError>
where
    P: FnMut(FollowsProgress),
{
    let mut cursor: Option<String> = None;
    let mut follows = Vec::new();
    let mut page_number = 0;

    loop {
        page_number += 1;
        progress(FollowsProgress::FetchingFollowsPage { page: page_number });
        let mut query = vec![("actor", actor.to_string()), ("limit", "100".to_string())];
        if let Some(cursor) = &cursor {
            query.push(("cursor", cursor.clone()));
        }

        let page = client
            .public_xrpc_query::<GetFollowsOutput>(FOLLOWS_METHOD, &query)
            .await?;
        follows.extend(page.follows);
        progress(FollowsProgress::FetchedFollowsPage { page: page_number, total: follows.len() });

        let Some(next_cursor) = page.cursor else {
            break;
        };

        if next_cursor.is_empty() {
            break;
        }

        cursor = Some(next_cursor);
    }

    Ok(follows)
}

async fn fetch_follows_last_posts(
    client: AtprotoClient, follows: Vec<super::ActorDefsProfileView>, progress: &mut impl FnMut(FollowsProgress),
) -> Result<Vec<FollowLastPost>, FollowsReportError> {
    run_parallel_rate_limited_with_progress(
        follows,
        ParallelConfig {
            max_parallel: LAST_POST_MAX_PARALLEL,
            start_delay: Duration::from_millis(LAST_POST_START_DELAY_MS),
        },
        move |follow| {
            let client = client.clone();
            async move {
                let last_post = fetch_last_post(&client, &follow.did).await?;
                Ok(FollowLastPost {
                    profile_url: profile_url(&follow.handle),
                    handle: follow.handle,
                    did: follow.did,
                    last_post_at: last_post.as_ref().map(|post| post.created_at.clone()),
                    last_post_rkey: last_post.as_ref().map(|post| post.rkey.clone()),
                    last_post_url: last_post.as_ref().map(|post| post.url.clone()),
                })
            }
        },
        |parallel| {
            progress(FollowsProgress::FetchingLastPosts { completed: parallel.completed, total: parallel.total });
        },
    )
    .await
    .map_err(FollowsReportError::ParallelFetch)
}

#[derive(Debug, Clone)]
struct LastPost {
    created_at: String,
    rkey: String,
    url: String,
}

async fn fetch_last_post(client: &AtprotoClient, actor: &str) -> Result<Option<LastPost>, ClientError> {
    let query = vec![
        ("actor", actor.to_string()),
        ("limit", "10".to_string()),
        ("filter", "posts_with_replies".to_string()),
        ("includePins", "false".to_string()),
    ];
    let feed = client
        .public_xrpc_query::<GetAuthorFeedOutput>(AUTHOR_FEED_METHOD, &query)
        .await?;

    Ok(feed.feed.into_iter().find_map(last_post_from_feed_item))
}

fn last_post_from_feed_item(item: FeedDefsFeedViewPost) -> Option<LastPost> {
    if item.reason.is_some() {
        return None;
    }

    let created_at = item
        .post
        .record
        .get("createdAt")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&item.post.indexed_at)
        .to_string();
    let rkey = rkey_from_at_uri(&item.post.uri)?;
    let handle = if item.post.author.handle.is_empty() { item.post.author.did } else { item.post.author.handle };

    Some(LastPost { created_at, rkey: rkey.clone(), url: post_url(&handle, &rkey) })
}

fn cache_key(did: &str, handle: &str, follows_count: Option<u64>, indexed_at: Option<&str>) -> String {
    let mut hash = Sha256::new();
    hash.update([CACHE_VERSION]);
    hash.update(did.as_bytes());
    hash.update([0]);
    hash.update(handle.as_bytes());
    hash.update([0]);
    hash.update(follows_count.unwrap_or_default().to_string().as_bytes());
    hash.update([0]);
    hash.update(indexed_at.unwrap_or_default().as_bytes());
    hex_digest(hash.finalize().as_slice())
}

fn cache_path(did: &str, cache_key: &str) -> Result<PathBuf, FollowsReportError> {
    let base = cache_base_dir().ok_or(FollowsReportError::MissingCacheDir)?;
    Ok(base
        .join("bsky-follows")
        .join(format!("{}-{cache_key}.json", safe_file_component(did))))
}

fn cache_base_dir() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|base| base.join("atproto-tools"))
}

fn read_cache(path: &Path) -> Result<Option<CacheFile>, FollowsReportError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)
        .map_err(|source| FollowsReportError::ReadCache { path: path.to_path_buf(), source })?;
    let cache = serde_json::from_str::<CacheFile>(&contents)
        .map_err(|source| FollowsReportError::ParseCache { path: path.to_path_buf(), source })?;

    if cache.version == CACHE_VERSION { Ok(Some(cache)) } else { Ok(None) }
}

fn write_cache(path: &Path, cache: &CacheFile) -> Result<(), FollowsReportError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| FollowsReportError::CreateCacheDir { path: parent.to_path_buf(), source })?;
    }

    let contents = serde_json::to_string_pretty(cache).map_err(FollowsReportError::SerializeCache)?;
    fs::write(path, format!("{contents}\n"))
        .map_err(|source| FollowsReportError::WriteCache { path: path.to_path_buf(), source })
}

impl CacheFile {
    fn into_report(self, cache_path: PathBuf) -> FollowsReport {
        FollowsReport {
            actor: self.actor,
            actor_did: self.actor_did,
            cache_key: self.cache_key,
            cache_path,
            generated_at_unix: self.generated_at_unix,
            follows: self.follows,
        }
    }
}

fn profile_url(handle: &str) -> String {
    format!("https://bsky.app/profile/{handle}")
}

fn post_url(handle: &str, rkey: &str) -> String {
    format!("https://bsky.app/profile/{handle}/post/{rkey}")
}

fn rkey_from_at_uri(uri: &str) -> Option<String> {
    uri.rsplit('/')
        .next()
        .filter(|rkey| !rkey.is_empty())
        .map(str::to_string)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn safe_file_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum FollowsReportError {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error("failed to fetch follow posts in parallel: {0}")]
    ParallelFetch(ParallelTaskError<ClientError>),
    #[error("could not determine cache directory; set XDG_CACHE_HOME or HOME")]
    MissingCacheDir,
    #[error("failed to create cache directory at {path}: {source}")]
    CreateCacheDir { path: PathBuf, source: std::io::Error },
    #[error("failed to read cache at {path}: {source}")]
    ReadCache { path: PathBuf, source: std::io::Error },
    #[error("failed to parse cache at {path}: {source}")]
    ParseCache { path: PathBuf, source: serde_json::Error },
    #[error("failed to serialize cache: {0}")]
    SerializeCache(serde_json::Error),
    #[error("failed to write cache at {path}: {source}")]
    WriteCache { path: PathBuf, source: std::io::Error },
}
