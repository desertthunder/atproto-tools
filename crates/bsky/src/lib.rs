// TODO: link_digest and follow - last post should be in an 'actions' module.
// - actions/fetch_links_for_digest.rs
// - actions/fetch_follow_last_post.rs

mod author_feed;
mod errors;
pub mod follows;
pub mod generated;
pub mod link_digest;

pub use errors::FollowsReportError;
pub use follows::{
    ActorTopLevelPost, Follow, FollowLastPost, FollowSync, FollowSyncOptions, FollowsOptions, FollowsProgress,
    FollowsReport, FollowsSort, FollowsSortDirection, FollowsSortField, fetch_actor_top_level_last_post,
    fetch_follow_sync, fetch_follow_sync_with_progress, fetch_follows_report, fetch_follows_report_with_progress,
    hash_follows,
};
pub use generated::*;
pub use link_digest::{
    AuthorFeedLinkOptions, ExternalLinkPost, fetch_follow_external_link_posts, fetch_follows_external_link_posts,
    fetch_follows_external_link_posts_with_progress,
};
