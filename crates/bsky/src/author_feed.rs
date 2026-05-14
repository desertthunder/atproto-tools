use super::{FeedDefsFeedViewPost, GetAuthorFeedOutput};
use atp_tools_core::{AtprotoClient, ClientError};

// TODO: sync https://github.com/bluesky-social/atproto/blob/main/lexicons/app/bsky/feed/getAuthorFeed.json
const AUTHOR_FEED_METHOD: &str = "app.bsky.feed.getAuthorFeed";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AuthorFeedFetchOptions {
    pub limit: u16,
    pub max_pages: usize,
    pub filter: AuthorFeedFilter,
    pub include_pins: bool,
}

impl Default for AuthorFeedFetchOptions {
    fn default() -> Self {
        Self { limit: 100, max_pages: 5, filter: AuthorFeedFilter::PostsWithReplies, include_pins: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorFeedFilter {
    PostsNoReplies,
    PostsWithReplies,
}

impl AuthorFeedFilter {
    fn as_str(self) -> &'static str {
        match self {
            Self::PostsNoReplies => "posts_no_replies",
            Self::PostsWithReplies => "posts_with_replies",
        }
    }
}

pub(crate) async fn fetch_author_feed(
    client: &AtprotoClient, actor: &str, options: AuthorFeedFetchOptions,
) -> Result<Vec<FeedDefsFeedViewPost>, ClientError> {
    let mut cursor: Option<String> = None;
    let mut feed = Vec::new();

    for _ in 0..options.max_pages {
        let mut query = vec![
            ("actor", actor.to_string()),
            ("limit", options.limit.min(100).to_string()),
            ("filter", options.filter.as_str().to_string()),
            ("includePins", options.include_pins.to_string()),
        ];
        if let Some(cursor) = &cursor {
            query.push(("cursor", cursor.clone()));
        }

        let page = client
            .public_xrpc_query::<GetAuthorFeedOutput>(AUTHOR_FEED_METHOD, &query)
            .await?;
        feed.extend(page.feed);

        let Some(next_cursor) = page.cursor else {
            break;
        };
        if next_cursor.is_empty() {
            break;
        }
        cursor = Some(next_cursor);
    }

    Ok(feed)
}
