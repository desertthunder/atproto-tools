use super::{FollowsReportError, GetAuthorFeedOutput, GetFollowsOutput};
use atp_tools_core::run_parallel_rate_limited_with_progress;
use atp_tools_core::{AtprotoClient, ClientError, ParallelConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_VERSION: u8 = 2;
const FOLLOWS_METHOD: &str = "app.bsky.graph.getFollows";
const AUTHOR_FEED_METHOD: &str = "app.bsky.feed.getAuthorFeed";
const LAST_POST_MAX_PARALLEL: usize = 8;
const LAST_POST_START_DELAY_MS: u64 = 50;
const AUTHOR_FEED_LIMIT: u16 = 100;
const AUTHOR_FEED_MAX_PAGES: usize = 5;
const HEX: &[u8; 16] = b"0123456789abcdef";

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
}

impl Default for FollowsOptions {
    fn default() -> Self {
        Self { limit: None, sort: FollowsSort::default() }
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
    ApplyingLimit { limit: usize },
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
    fetch_follows_report_with_progress(client, actor, refresh, FollowsOptions::default(), |_| {}).await
}

pub async fn fetch_follows_report_with_progress<P>(
    client: &AtprotoClient, actor: &str, refresh: bool, options: FollowsOptions, mut progress: P,
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
        options.limit,
    );

    let cache_path = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|base| base.join("atproto-tools"))
        .ok_or(FollowsReportError::MissingCacheDir)?
        .join("bsky-follows")
        .join(format!(
            "{}-{cache_key}.json",
            profile
                .did
                .chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
                .collect::<String>()
        ));

    if !refresh {
        progress(FollowsProgress::CheckingCache { path: cache_path.clone() });
        if let Some(report) = read_cache(&cache_path)? {
            progress(FollowsProgress::CacheHit { path: cache_path.clone(), count: report.follows.len() });
            let mut report = report.into_report(cache_path);
            sort_report(&mut report, options.sort);
            return Ok(report);
        }
    }

    if let Some(limit) = options.limit {
        progress(FollowsProgress::ApplyingLimit { limit });
    }

    let follows = fetch_all_follows(client, actor, options.limit, &mut progress).await?;
    let rows = fetch_follows_last_posts(client.clone(), follows, &mut progress).await?;

    let cache_file = CacheFile {
        version: CACHE_VERSION,
        actor: profile.handle,
        actor_did: profile.did,
        cache_key,
        generated_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        follows: rows,
    };

    progress(FollowsProgress::WritingCache { path: cache_path.clone() });
    write_cache(&cache_path, &cache_file)?;
    progress(FollowsProgress::WroteCache { path: cache_path.clone() });
    let mut report = cache_file.into_report(cache_path);
    sort_report(&mut report, options.sort);
    Ok(report)
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

fn cache_key(
    did: &str, handle: &str, follows_count: Option<u64>, indexed_at: Option<&str>, limit: Option<usize>,
) -> String {
    let mut hash = Sha256::new();
    hash.update([CACHE_VERSION]);
    hash.update(did.as_bytes());
    hash.update([0]);
    hash.update(handle.as_bytes());
    hash.update([0]);
    hash.update(follows_count.unwrap_or_default().to_string().as_bytes());
    hash.update([0]);
    hash.update(indexed_at.unwrap_or_default().as_bytes());
    hash.update([0]);
    hash.update(limit.map(|limit| limit.to_string()).unwrap_or_default().as_bytes());

    let bytes = hash.finalize();
    let mut output = String::with_capacity(bytes.len() * 2);
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

fn read_cache(path: &Path) -> Result<Option<CacheFile>, FollowsReportError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|source| FollowsReportError::ReadCache { path: path.to_path_buf(), source })?;
    let cache = serde_json::from_str::<CacheFile>(&contents)
        .map_err(|source| FollowsReportError::ParseCache { path: path.to_path_buf(), source })?;

    if cache.version == CACHE_VERSION { Ok(Some(cache)) } else { Ok(None) }
}

fn write_cache(path: &Path, cache: &CacheFile) -> Result<(), FollowsReportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| FollowsReportError::CreateCacheDir { path: parent.to_path_buf(), source })?;
    }

    let contents = serde_json::to_string_pretty(cache).map_err(FollowsReportError::SerializeCache)?;
    std::fs::write(path, format!("{contents}\n"))
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

    fn report(follows: Vec<FollowLastPost>) -> FollowsReport {
        FollowsReport {
            actor: "actor.test".to_string(),
            actor_did: "did:plc:actor".to_string(),
            cache_key: "cache".to_string(),
            cache_path: PathBuf::from("cache.json"),
            generated_at_unix: 0,
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
}
