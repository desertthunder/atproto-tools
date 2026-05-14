use super::{FeedDefsPostView, Follow, GetAuthorFeedOutput};
use atp_tools_core::{AtprotoClient, ClientError, ParallelConfig, ParallelTaskError, run_parallel_rate_limited};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const AUTHOR_FEED_METHOD: &str = "app.bsky.feed.getAuthorFeed";
const AUTHOR_FEED_LIMIT: u16 = 100;
const AUTHOR_FEED_MAX_PAGES: usize = 5;
const AUTHOR_FEED_MAX_PARALLEL: usize = 4;
const AUTHOR_FEED_START_DELAY_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLinkPost {
    pub author: String,
    pub author_did: String,
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
    let mut cursor: Option<String> = None;
    let mut links = Vec::new();

    for _ in 0..options.max_pages {
        let mut query = vec![
            ("actor", follow.did.clone()),
            ("filter", "posts_with_replies".to_string()),
            ("includePins", "false".to_string()),
            ("limit", options.limit.min(AUTHOR_FEED_LIMIT).to_string()),
        ];
        if let Some(cursor) = &cursor {
            query.push(("cursor", cursor.clone()));
        }

        let page = client
            .public_xrpc_query::<GetAuthorFeedOutput>(AUTHOR_FEED_METHOD, &query)
            .await?;
        links.extend(
            page.feed
                .iter()
                .map(|item| &item.post)
                .filter(|post| post_matches_window(post, options))
                .filter_map(extract_external_link_post),
        );

        let Some(next_cursor) = page.cursor else {
            break;
        };
        if next_cursor.is_empty() {
            break;
        }
        cursor = Some(next_cursor);
    }

    Ok(links)
}

pub async fn fetch_follows_external_link_posts(
    client: AtprotoClient, follows: Vec<Follow>, options: AuthorFeedLinkOptions,
) -> Result<Vec<ExternalLinkPost>, ParallelTaskError<ClientError>> {
    let mut nested = run_parallel_rate_limited(
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

fn extract_external_link_post(post: &FeedDefsPostView) -> Option<ExternalLinkPost> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_external_embed_link_post() {
        let post = serde_json::from_value::<FeedDefsPostView>(serde_json::json!({
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
        }))
        .expect("post view");

        let link = extract_external_link_post(&post).expect("external link");

        assert_eq!(link.author, "author.test");
        assert_eq!(link.author_did, "did:plc:author");
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
        let post = serde_json::from_value::<FeedDefsPostView>(serde_json::json!({
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
        }))
        .expect("post view");

        assert_eq!(extract_external_link_post(&post), None);
    }
}
