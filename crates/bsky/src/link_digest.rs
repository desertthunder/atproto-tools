use super::{
    FeedDefsFeedViewPost, FeedDefsPostView, Follow,
    author_feed::{AuthorFeedFetchOptions, AuthorFeedFilter, fetch_author_feed},
};
use atp_tools_core::{
    AtprotoClient, ClientError, ParallelConfig, ParallelProgress, ParallelTaskError,
    run_parallel_rate_limited_with_progress,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const AUTHOR_FEED_LIMIT: u16 = 100;
const AUTHOR_FEED_MAX_PAGES: usize = 5;
const AUTHOR_FEED_MAX_PARALLEL: usize = 4;
const AUTHOR_FEED_START_DELAY_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLinkPost {
    pub author: String,
    pub author_did: String,
    pub shared_by: String,
    pub shared_by_did: String,
    pub shared_at: String,
    pub post_uri: String,
    pub indexed_at: String,
    pub created_at: Option<String>,
    pub external_uri: String,
    pub title: String,
    pub description: String,
    pub bookmark_count: i64,
    pub repost_count: i64,
    pub like_count: i64,
}

impl ExternalLinkPost {
    pub fn score(&self) -> i64 {
        self.bookmark_count + self.repost_count + self.like_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorFeedLinkOptions {
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: u16,
    pub max_pages: usize,
}

impl Default for AuthorFeedLinkOptions {
    fn default() -> Self {
        Self { since: None, until: None, limit: AUTHOR_FEED_LIMIT, max_pages: AUTHOR_FEED_MAX_PAGES }
    }
}

pub async fn fetch_follow_external_link_posts(
    client: &AtprotoClient, follow: &Follow, options: &AuthorFeedLinkOptions,
) -> Result<Vec<ExternalLinkPost>, ClientError> {
    let mut links = Vec::new();
    let feed = fetch_author_feed(
        client,
        &follow.did,
        AuthorFeedFetchOptions {
            limit: options.limit,
            max_pages: options.max_pages,
            filter: AuthorFeedFilter::PostsWithReplies,
            include_pins: false,
        },
    )
    .await?;
    links.extend(
        feed.iter()
            .filter(|item| post_matches_window(&item.post, options))
            .filter_map(|item| extract_external_link_post(item, follow)),
    );

    Ok(links)
}

pub async fn fetch_follows_external_link_posts(
    client: AtprotoClient, follows: Vec<Follow>, options: AuthorFeedLinkOptions,
) -> Result<Vec<ExternalLinkPost>, ParallelTaskError<ClientError>> {
    fetch_follows_external_link_posts_with_progress(client, follows, options, |_| {}).await
}

pub async fn fetch_follows_external_link_posts_with_progress<P>(
    client: AtprotoClient, follows: Vec<Follow>, options: AuthorFeedLinkOptions, progress: P,
) -> Result<Vec<ExternalLinkPost>, ParallelTaskError<ClientError>>
where
    P: FnMut(ParallelProgress),
{
    let mut nested = run_parallel_rate_limited_with_progress(
        follows,
        ParallelConfig {
            max_parallel: AUTHOR_FEED_MAX_PARALLEL,
            start_delay: Duration::from_millis(AUTHOR_FEED_START_DELAY_MS),
        },
        move |follow| {
            let client = client.clone();
            let options = options.clone();
            async move { fetch_follow_external_link_posts(&client, &follow, &options).await }
        },
        progress,
    )
    .await?;

    Ok(nested.drain(..).flatten().collect())
}

fn post_matches_window(post: &FeedDefsPostView, options: &AuthorFeedLinkOptions) -> bool {
    let sort_at = post
        .record
        .get("createdAt")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&post.indexed_at);

    if options.since.as_deref().is_some_and(|since| sort_at < since) {
        return false;
    }

    if options.until.as_deref().is_some_and(|until| sort_at >= until) {
        return false;
    }

    true
}

fn extract_external_link_post(item: &FeedDefsFeedViewPost, follow: &Follow) -> Option<ExternalLinkPost> {
    let post = &item.post;
    let embed = post.record.get("embed")?;
    if embed.get("$type")?.as_str()? != "app.bsky.embed.external" {
        return None;
    }

    let external = embed.get("external")?;
    let external_uri = external.get("uri")?.as_str()?.to_string();
    let title = external
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let description = external
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let created_at = post
        .record
        .get("createdAt")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    Some(ExternalLinkPost {
        author: post.author.handle.clone(),
        author_did: post.author.did.clone(),
        shared_by: follow.handle.clone(),
        shared_by_did: follow.did.clone(),
        shared_at: shared_at(item),
        post_uri: post.uri.clone(),
        indexed_at: post.indexed_at.clone(),
        created_at,
        external_uri,
        title,
        description,
        bookmark_count: post.bookmark_count.unwrap_or_default(),
        repost_count: post.repost_count.unwrap_or_default(),
        like_count: post.like_count.unwrap_or_default(),
    })
}

fn shared_at(item: &FeedDefsFeedViewPost) -> String {
    item.reason
        .as_ref()
        .and_then(|reason| reason.get("indexedAt"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| item.post.record.get("createdAt").and_then(serde_json::Value::as_str))
        .unwrap_or(&item.post.indexed_at)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_external_embed_link_post() {
        let item = feed_item(serde_json::json!({
            "uri": "at://did:plc:author/app.bsky.feed.post/abc",
            "cid": "bafy",
            "author": {
                "did": "did:plc:author",
                "handle": "author.test"
            },
            "record": {
                "$type": "app.bsky.feed.post",
                "createdAt": "2026-05-14T01:55:12.540Z",
                "embed": {
                    "$type": "app.bsky.embed.external",
                    "external": {
                        "description": "A rounded display face.",
                        "title": "Sniglet - Google Fonts",
                        "uri": "https://fonts.google.com/specimen/Sniglet"
                    }
                },
                "text": "fonts.google.com/specimen/Sni..."
            },
            "bookmarkCount": 2,
            "indexedAt": "2026-05-14T01:55:14.174Z",
            "likeCount": 5,
            "repostCount": 3
        }));
        let follow = follow("sharer.test");

        let link = extract_external_link_post(&item, &follow).expect("external link");

        assert_eq!(link.author, "author.test");
        assert_eq!(link.author_did, "did:plc:author");
        assert_eq!(link.shared_by, "sharer.test");
        assert_eq!(link.shared_by_did, "did:plc:sharer.test");
        assert_eq!(link.shared_at, "2026-05-14T01:55:12.540Z");
        assert_eq!(link.post_uri, "at://did:plc:author/app.bsky.feed.post/abc");
        assert_eq!(link.created_at.as_deref(), Some("2026-05-14T01:55:12.540Z"));
        assert_eq!(link.external_uri, "https://fonts.google.com/specimen/Sniglet");
        assert_eq!(link.title, "Sniglet - Google Fonts");
        assert_eq!(link.description, "A rounded display face.");
        assert_eq!(link.bookmark_count, 2);
        assert_eq!(link.repost_count, 3);
        assert_eq!(link.like_count, 5);
    }

    #[test]
    fn skips_non_external_embeds() {
        let item = feed_item(serde_json::json!({
            "uri": "at://did:plc:author/app.bsky.feed.post/abc",
            "cid": "bafy",
            "author": {
                "did": "did:plc:author",
                "handle": "author.test"
            },
            "record": {
                "$type": "app.bsky.feed.post",
                "embed": {
                    "$type": "app.bsky.embed.images",
                    "images": []
                },
                "text": "image post"
            },
            "indexedAt": "2026-05-14T01:55:14.174Z"
        }));

        assert_eq!(extract_external_link_post(&item, &follow("sharer.test")), None);
    }

    #[test]
    fn scores_link_posts_from_bookmarks_reposts_and_likes() {
        let link = ExternalLinkPost {
            author: "author.test".to_string(),
            author_did: "did:plc:author".to_string(),
            shared_by: "sharer.test".to_string(),
            shared_by_did: "did:plc:sharer".to_string(),
            shared_at: "2026-05-14T01:55:12.540Z".to_string(),
            post_uri: "at://did:plc:author/app.bsky.feed.post/abc".to_string(),
            indexed_at: "2026-05-14T01:55:14.174Z".to_string(),
            created_at: Some("2026-05-14T01:55:12.540Z".to_string()),
            external_uri: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: "Example description".to_string(),
            bookmark_count: 2,
            repost_count: 3,
            like_count: 5,
        };

        assert_eq!(link.score(), 10);
    }

    fn feed_item(post: serde_json::Value) -> FeedDefsFeedViewPost {
        serde_json::from_value::<FeedDefsFeedViewPost>(serde_json::json!({ "post": post })).expect("feed item")
    }

    fn follow(handle: &str) -> Follow {
        Follow {
            handle: handle.to_string(),
            did: format!("did:plc:{handle}"),
            profile_url: format!("https://bsky.app/profile/{handle}"),
        }
    }
}
