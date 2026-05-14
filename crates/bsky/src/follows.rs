use super::{FollowsReportError, GetAuthorFeedOutput, GetFollowsOutput};
use atp_tools_core::run_parallel_rate_limited_with_progress;
use atp_tools_core::{AtprotoClient, ClientError, ParallelConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPORT_CACHE_VERSION: u8 = 2;
const FOLLOW_SYNC_CACHE_VERSION: u8 = 1;
const FOLLOWS_METHOD: &str = "app.bsky.graph.getFollows";
const AUTHOR_FEED_METHOD: &str = "app.bsky.feed.getAuthorFeed";
const LAST_POST_MAX_PARALLEL: usize = 8;
const LAST_POST_START_DELAY_MS: u64 = 50;
const AUTHOR_FEED_LIMIT: u16 = 100;
const AUTHOR_FEED_MAX_PAGES: usize = 5;
const HEX: &[u8; 16] = b"0123456789abcdef";
const DEFAULT_FOLLOW_RECACHE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowSync {
    pub actor: String,
    pub actor_did: String,
    pub cache_path: PathBuf,
    pub generated_at_unix: u64,
    pub follows_hash: String,
    pub follows: Vec<Follow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub handle: String,
    pub did: String,
    pub profile_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowsReport {
    pub actor: String,
    pub actor_did: String,
    pub cache_key: String,
    pub cache_path: PathBuf,
    pub generated_at_unix: u64,
    pub follows_hash: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorTopLevelPost {
    pub created_at: String,
    pub uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowsOptions {
    pub limit: Option<usize>,
    pub sort: FollowsSort,
    pub recache_after: Option<Duration>,
}

impl Default for FollowsOptions {
    fn default() -> Self {
        Self { limit: None, sort: FollowsSort::default(), recache_after: Some(DEFAULT_FOLLOW_RECACHE_AFTER) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowSyncOptions {
    pub limit: Option<usize>,
    pub recache_after: Option<Duration>,
}

impl Default for FollowSyncOptions {
    fn default() -> Self {
        Self { limit: None, recache_after: Some(DEFAULT_FOLLOW_RECACHE_AFTER) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowsSort {
    pub field: FollowsSortField,
    pub direction: FollowsSortDirection,
}

impl Default for FollowsSort {
    fn default() -> Self {
        Self { field: FollowsSortField::LastPostAt, direction: FollowsSortDirection::Asc }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowsSortField {
    Handle,
    Did,
    ProfileUrl,
    LastPostAt,
    LastPostRkey,
    LastPostUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowsSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowsProgress {
    ResolvingActor,
    CheckingCache { path: PathBuf },
    CacheHit { path: PathBuf, count: usize },
    CacheStale { path: PathBuf, generated_at_unix: u64 },
    ApplyingLimit { limit: usize },
    FetchingFollowsPage { page: usize },
    FetchedFollowsPage { page: usize, total: usize },
    FetchingLastPosts { completed: usize, total: usize },
    WritingCache { path: PathBuf },
    WroteCache { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FollowSyncCacheFile {
    version: u8,
    actor: String,
    actor_did: String,
    generated_at_unix: u64,
    follows_hash: String,
    follows: Vec<Follow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportCacheFile {
    version: u8,
    actor: String,
    actor_did: String,
    cache_key: String,
    generated_at_unix: u64,
    follows_hash: String,
    follows: Vec<FollowLastPost>,
}

pub async fn fetch_follows_report(
    client: &AtprotoClient, actor: &str, refresh: bool,
) -> Result<FollowsReport, FollowsReportError> {
    fetch_follows_report_with_progress(client, actor, refresh, FollowsOptions::default(), |_| {}).await
}

pub async fn fetch_follows_report_with_progress<P>(
    client: &AtprotoClient, actor: &str, refresh: bool, options: FollowsOptions, mut progress: P,
) -> Result<FollowsReport, FollowsReportError>
where
    P: FnMut(FollowsProgress) + Send,
{
    let sync = fetch_follow_sync_with_progress(
        client,
        actor,
        refresh,
        FollowSyncOptions { limit: options.limit, recache_after: options.recache_after },
        &mut progress,
    )
    .await?;
    let cache_key = {
        let mut hash = Sha256::new();
        hash.update([REPORT_CACHE_VERSION]);
        hash.update(sync.actor_did.as_bytes());
        hash.update([0]);
        hash.update(sync.follows_hash.as_bytes());
        hex_hash(hash.finalize())
    };
    let cache_path = cache_base_dir()?.join("bsky-follows").join(format!(
        "{}-{cache_key}.json",
        sanitize_cache_component(&sync.actor_did)
    ));

    if !refresh {
        progress(FollowsProgress::CheckingCache { path: cache_path.clone() });
        if let Some(report) = read_report_cache(&cache_path)? {
            progress(FollowsProgress::CacheHit { path: cache_path.clone(), count: report.follows.len() });
            let mut report = report.into_report(cache_path);
            sort_report(&mut report, options.sort);
            return Ok(report);
        }
    }

    let rows = fetch_follows_last_posts(client.clone(), sync.follows.clone(), &mut progress).await?;

    let cache_file = ReportCacheFile {
        version: REPORT_CACHE_VERSION,
        actor: sync.actor,
        actor_did: sync.actor_did,
        cache_key,
        generated_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        follows_hash: sync.follows_hash,
        follows: rows,
    };

    progress(FollowsProgress::WritingCache { path: cache_path.clone() });
    write_report_cache(&cache_path, &cache_file)?;
    progress(FollowsProgress::WroteCache { path: cache_path.clone() });
    let mut report = cache_file.into_report(cache_path);
    sort_report(&mut report, options.sort);
    Ok(report)
}

pub async fn fetch_follow_sync(
    client: &AtprotoClient, actor: &str, refresh: bool,
) -> Result<FollowSync, FollowsReportError> {
    fetch_follow_sync_with_progress(client, actor, refresh, FollowSyncOptions::default(), |_| {}).await
}

pub async fn fetch_follow_sync_with_progress<P>(
    client: &AtprotoClient, actor: &str, refresh: bool, options: FollowSyncOptions, mut progress: P,
) -> Result<FollowSync, FollowsReportError>
where
    P: FnMut(FollowsProgress),
{
    progress(FollowsProgress::ResolvingActor);
    let profile = client.get_profile(actor).await?;
    let cache_path = follow_sync_cache_path(&profile.did, options.limit)?;

    if !refresh {
        progress(FollowsProgress::CheckingCache { path: cache_path.clone() });
        if let Some(cache) = read_follow_sync_cache(&cache_path)? {
            if cache_is_fresh(cache.generated_at_unix, options.recache_after) {
                progress(FollowsProgress::CacheHit { path: cache_path.clone(), count: cache.follows.len() });
                return Ok(cache.into_sync(cache_path));
            }

            progress(FollowsProgress::CacheStale {
                path: cache_path.clone(),
                generated_at_unix: cache.generated_at_unix,
            });
        }
    }

    if let Some(limit) = options.limit {
        progress(FollowsProgress::ApplyingLimit { limit });
    }

    let follows = fetch_all_follows(client, actor, options.limit, &mut progress)
        .await?
        .into_iter()
        .map(|follow| Follow { profile_url: profile_url(&follow.handle), handle: follow.handle, did: follow.did })
        .collect::<Vec<_>>();
    let follows_hash = hash_follows(&follows);
    let cache_file = FollowSyncCacheFile {
        version: FOLLOW_SYNC_CACHE_VERSION,
        actor: profile.handle,
        actor_did: profile.did,
        generated_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        follows_hash,
        follows,
    };

    progress(FollowsProgress::WritingCache { path: cache_path.clone() });
    write_follow_sync_cache(&cache_path, &cache_file)?;
    progress(FollowsProgress::WroteCache { path: cache_path.clone() });
    Ok(cache_file.into_sync(cache_path))
}

async fn fetch_all_follows<P>(
    client: &AtprotoClient, actor: &str, limit: Option<usize>, progress: &mut P,
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
        if let Some(limit) = limit {
            follows.truncate(limit);
        }
        progress(FollowsProgress::FetchedFollowsPage { page: page_number, total: follows.len() });

        if limit.is_some_and(|limit| follows.len() >= limit) {
            break;
        }

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
    client: AtprotoClient, follows: Vec<Follow>, progress: &mut impl FnMut(FollowsProgress),
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

pub async fn fetch_actor_top_level_last_post(
    client: &AtprotoClient, actor: &str,
) -> Result<Option<ActorTopLevelPost>, ClientError> {
    let query = vec![
        ("actor", actor.to_string()),
        ("limit", "1".to_string()),
        ("filter", "posts_no_replies".to_string()),
        ("includePins", "false".to_string()),
    ];
    let feed = client
        .public_xrpc_query::<GetAuthorFeedOutput>(AUTHOR_FEED_METHOD, &query)
        .await?;

    Ok(feed.feed.into_iter().find_map(|item| {
        if item.reason.is_some() || item.reply.is_some() || item.post.author.did != actor {
            return None;
        }

        let created_at = item
            .post
            .record
            .get("createdAt")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&item.post.indexed_at)
            .to_string();

        Some(ActorTopLevelPost { created_at, uri: item.post.uri })
    }))
}

async fn fetch_last_post(client: &AtprotoClient, actor: &str) -> Result<Option<LastPost>, ClientError> {
    let mut cursor: Option<String> = None;

    for _ in 0..AUTHOR_FEED_MAX_PAGES {
        let mut query = vec![
            ("actor", actor.to_string()),
            ("limit", AUTHOR_FEED_LIMIT.to_string()),
            ("filter", "posts_with_replies".to_string()),
            ("includePins", "false".to_string()),
        ];
        if let Some(cursor) = &cursor {
            query.push(("cursor", cursor.clone()));
        }

        let feed = client
            .public_xrpc_query::<GetAuthorFeedOutput>(AUTHOR_FEED_METHOD, &query)
            .await?;

        if let Some(post) = feed.feed.into_iter().find_map(|item| {
            if item.reason.is_some() || item.post.author.did != actor {
                return None;
            }

            let created_at = item
                .post
                .record
                .get("createdAt")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&item.post.indexed_at)
                .to_string();
            let rkey = item
                .post
                .uri
                .rsplit('/')
                .next()
                .filter(|rkey| !rkey.is_empty())
                .map(str::to_string)?;
            let handle =
                if item.post.author.handle.is_empty() { item.post.author.did } else { item.post.author.handle };

            Some(LastPost {
                created_at,
                rkey: rkey.clone(),
                url: format!("https://bsky.app/profile/{handle}/post/{rkey}"),
            })
        }) {
            return Ok(Some(post));
        }

        let Some(next_cursor) = feed.cursor else {
            break;
        };

        if next_cursor.is_empty() {
            break;
        }

        cursor = Some(next_cursor);
    }

    Ok(None)
}

pub fn hash_follows(follows: &[Follow]) -> String {
    let mut normalized = follows
        .iter()
        .map(|follow| (follow.did.as_str(), follow.handle.as_str()))
        .collect::<Vec<_>>();
    normalized.sort_unstable();

    let mut hash = Sha256::new();
    hash.update([FOLLOW_SYNC_CACHE_VERSION]);
    for (did, handle) in normalized {
        hash.update(did.as_bytes());
        hash.update([0]);
        hash.update(handle.as_bytes());
        hash.update([0]);
    }

    hex_hash(hash.finalize())
}

fn hex_hash(bytes: impl IntoIterator<Item = u8>) -> String {
    let mut output = String::new();
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn sort_report(report: &mut FollowsReport, sort: FollowsSort) {
    report.follows.sort_by(|left, right| {
        match sort.field {
            FollowsSortField::Handle => compare_required(&left.handle, &right.handle, sort.direction),
            FollowsSortField::Did => compare_required(&left.did, &right.did, sort.direction),
            FollowsSortField::ProfileUrl => compare_required(&left.profile_url, &right.profile_url, sort.direction),
            FollowsSortField::LastPostAt => compare_optional(
                left.last_post_at.as_deref(),
                right.last_post_at.as_deref(),
                sort.direction,
            ),
            FollowsSortField::LastPostRkey => compare_optional(
                left.last_post_rkey.as_deref(),
                right.last_post_rkey.as_deref(),
                sort.direction,
            ),
            FollowsSortField::LastPostUrl => compare_optional(
                left.last_post_url.as_deref(),
                right.last_post_url.as_deref(),
                sort.direction,
            ),
        }
        .then_with(|| left.handle.cmp(&right.handle))
        .then_with(|| left.did.cmp(&right.did))
    });
}

fn compare_required(left: &str, right: &str, direction: FollowsSortDirection) -> Ordering {
    match direction {
        FollowsSortDirection::Asc => left.cmp(right),
        FollowsSortDirection::Desc => right.cmp(left),
    }
}

fn compare_optional(left: Option<&str>, right: Option<&str>, direction: FollowsSortDirection) -> Ordering {
    match (
        left.filter(|value| !value.is_empty()),
        right.filter(|value| !value.is_empty()),
    ) {
        (Some(left), Some(right)) => compare_required(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn read_follow_sync_cache(path: &Path) -> Result<Option<FollowSyncCacheFile>, FollowsReportError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|source| FollowsReportError::ReadCache { path: path.to_path_buf(), source })?;
    let cache = serde_json::from_str::<FollowSyncCacheFile>(&contents)
        .map_err(|source| FollowsReportError::ParseCache { path: path.to_path_buf(), source })?;

    if cache.version == FOLLOW_SYNC_CACHE_VERSION { Ok(Some(cache)) } else { Ok(None) }
}

fn write_follow_sync_cache(path: &Path, cache: &FollowSyncCacheFile) -> Result<(), FollowsReportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| FollowsReportError::CreateCacheDir { path: parent.to_path_buf(), source })?;
    }

    let contents = serde_json::to_string_pretty(cache).map_err(FollowsReportError::SerializeCache)?;
    std::fs::write(path, format!("{contents}\n"))
        .map_err(|source| FollowsReportError::WriteCache { path: path.to_path_buf(), source })
}

fn read_report_cache(path: &Path) -> Result<Option<ReportCacheFile>, FollowsReportError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|source| FollowsReportError::ReadCache { path: path.to_path_buf(), source })?;
    let cache = serde_json::from_str::<ReportCacheFile>(&contents)
        .map_err(|source| FollowsReportError::ParseCache { path: path.to_path_buf(), source })?;

    if cache.version == REPORT_CACHE_VERSION { Ok(Some(cache)) } else { Ok(None) }
}

fn write_report_cache(path: &Path, cache: &ReportCacheFile) -> Result<(), FollowsReportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| FollowsReportError::CreateCacheDir { path: parent.to_path_buf(), source })?;
    }

    let contents = serde_json::to_string_pretty(cache).map_err(FollowsReportError::SerializeCache)?;
    std::fs::write(path, format!("{contents}\n"))
        .map_err(|source| FollowsReportError::WriteCache { path: path.to_path_buf(), source })
}

fn follow_sync_cache_path(did: &str, limit: Option<usize>) -> Result<PathBuf, FollowsReportError> {
    let suffix = limit
        .map(|limit| format!("limit-{limit}"))
        .unwrap_or_else(|| "all".to_string());
    Ok(cache_base_dir()?
        .join("bsky-follow-sync")
        .join(format!("{}-{suffix}.json", sanitize_cache_component(did))))
}

fn cache_base_dir() -> Result<PathBuf, FollowsReportError> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|base| base.join("atproto-tools"))
        .ok_or(FollowsReportError::MissingCacheDir)
}

fn sanitize_cache_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn cache_is_fresh(generated_at_unix: u64, recache_after: Option<Duration>) -> bool {
    let Some(recache_after) = recache_after else {
        return true;
    };
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return true;
    };

    now.as_secs().saturating_sub(generated_at_unix) < recache_after.as_secs()
}

impl FollowSyncCacheFile {
    fn into_sync(self, cache_path: PathBuf) -> FollowSync {
        FollowSync {
            actor: self.actor,
            actor_did: self.actor_did,
            cache_path,
            generated_at_unix: self.generated_at_unix,
            follows_hash: self.follows_hash,
            follows: self.follows,
        }
    }
}

impl ReportCacheFile {
    fn into_report(self, cache_path: PathBuf) -> FollowsReport {
        FollowsReport {
            actor: self.actor,
            actor_did: self.actor_did,
            cache_key: self.cache_key,
            cache_path,
            generated_at_unix: self.generated_at_unix,
            follows_hash: self.follows_hash,
            follows: self.follows,
        }
    }
}

fn profile_url(handle: &str) -> String {
    format!("https://bsky.app/profile/{handle}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_last_post_at_with_missing_values_last() {
        let mut report = report(vec![
            follow("newer", Some("2026-05-10T10:00:00.000Z")),
            follow("missing", None),
            follow("older", Some("2026-05-09T10:00:00.000Z")),
        ]);

        sort_report(
            &mut report,
            FollowsSort { field: FollowsSortField::LastPostAt, direction: FollowsSortDirection::Asc },
        );
        assert_eq!(handles(&report), ["older", "newer", "missing"]);

        sort_report(
            &mut report,
            FollowsSort { field: FollowsSortField::LastPostAt, direction: FollowsSortDirection::Desc },
        );
        assert_eq!(handles(&report), ["newer", "older", "missing"]);
    }

    #[test]
    fn sorts_handles_descending() {
        let mut report = report(vec![
            follow("alpha", None),
            follow("charlie", None),
            follow("bravo", None),
        ]);

        sort_report(
            &mut report,
            FollowsSort { field: FollowsSortField::Handle, direction: FollowsSortDirection::Desc },
        );

        assert_eq!(handles(&report), ["charlie", "bravo", "alpha"]);
    }

    #[test]
    fn hashes_follow_lists_by_content_not_order() {
        let left = vec![sync_follow("bravo"), sync_follow("alpha")];
        let right = vec![sync_follow("alpha"), sync_follow("bravo")];

        assert_eq!(hash_follows(&left), hash_follows(&right));
    }

    #[test]
    fn hashes_change_when_follow_list_changes() {
        let original = vec![sync_follow("alpha"), sync_follow("bravo")];
        let changed = vec![sync_follow("alpha"), sync_follow("charlie")];

        assert_ne!(hash_follows(&original), hash_follows(&changed));
    }

    fn report(follows: Vec<FollowLastPost>) -> FollowsReport {
        FollowsReport {
            actor: "actor.test".to_string(),
            actor_did: "did:plc:actor".to_string(),
            cache_key: "cache".to_string(),
            cache_path: PathBuf::from("cache.json"),
            generated_at_unix: 0,
            follows_hash: "hash".to_string(),
            follows,
        }
    }

    fn follow(handle: &str, last_post_at: Option<&str>) -> FollowLastPost {
        FollowLastPost {
            handle: handle.to_string(),
            did: format!("did:plc:{handle}"),
            profile_url: profile_url(handle),
            last_post_at: last_post_at.map(str::to_string),
            last_post_rkey: None,
            last_post_url: None,
        }
    }

    fn handles(report: &FollowsReport) -> Vec<&str> {
        report.follows.iter().map(|follow| follow.handle.as_str()).collect()
    }

    fn sync_follow(handle: &str) -> Follow {
        Follow { handle: handle.to_string(), did: format!("did:plc:{handle}"), profile_url: profile_url(handle) }
    }
}
