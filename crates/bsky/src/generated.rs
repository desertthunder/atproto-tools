// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsAdultContentPref {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsBskyAppProgressGuide {
    pub guide: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsBskyAppStatePref {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "activeProgressGuide")]
    pub active_progress_guide: Option<ActorDefsBskyAppProgressGuide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nuxs: Option<Vec<ActorDefsNux>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "queuedNudges")]
    pub queued_nudges: Option<Vec<std::string::String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsContentLabelPref {
    pub label: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "labelerDid")]
    pub labeler_did: Option<std::string::String>,
    pub visibility: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsDeclaredAgePref {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isOverAge13")]
    pub is_over_age13: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isOverAge16")]
    pub is_over_age16: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isOverAge18")]
    pub is_over_age18: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsFeedViewPref {
    pub feed: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideQuotePosts")]
    pub hide_quote_posts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideReplies")]
    pub hide_replies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideRepliesByLikeCount")]
    pub hide_replies_by_like_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideRepliesByUnfollowed")]
    pub hide_replies_by_unfollowed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideReposts")]
    pub hide_reposts: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsHiddenPostsPref {
    pub items: Vec<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsInterestsPref {
    pub tags: Vec<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsKnownFollowers {
    pub count: i64,
    pub followers: Vec<ActorDefsProfileViewBasic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsLabelerPrefItem {
    pub did: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsLabelersPref {
    pub labelers: Vec<ActorDefsLabelerPrefItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsLiveEventPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hiddenFeedIds")]
    pub hidden_feed_ids: Option<Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideAllFeeds")]
    pub hide_all_feeds: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsMutedWord {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "actorTarget")]
    pub actor_target: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<std::string::String>,
    pub targets: Vec<serde_json::Value>,
    pub value: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsMutedWordsPref {
    pub items: Vec<ActorDefsMutedWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsNux {
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<std::string::String>,
    pub id: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsPersonalDetailsPref {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "birthDate")]
    pub birth_date: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsPostInteractionSettingsPref {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "postgateEmbeddingRules")]
    pub postgate_embedding_rules: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "threadgateAllowRules")]
    pub threadgate_allow_rules: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileAssociated {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "activitySubscription")]
    pub activity_subscription: Option<ActorDefsProfileAssociatedActivitySubscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<ActorDefsProfileAssociatedChat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedgens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub germ: Option<ActorDefsProfileAssociatedGerm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labeler: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "starterPacks")]
    pub starter_packs: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileAssociatedActivitySubscription {
    #[serde(rename = "allowSubscriptions")]
    pub allow_subscriptions: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileAssociatedChat {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "allowGroupInvites")]
    pub allow_group_invites: Option<std::string::String>,
    #[serde(rename = "allowIncoming")]
    pub allow_incoming: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileAssociatedGerm {
    #[serde(rename = "messageMeUrl")]
    pub message_me_url: std::string::String,
    #[serde(rename = "showButtonTo")]
    pub show_button_to: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<ActorDefsProfileAssociated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<std::string::String>,
    pub did: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<std::string::String>,
    pub handle: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "indexedAt")]
    pub indexed_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ActorDefsStatusView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<ActorDefsVerificationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ActorDefsViewerState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileViewBasic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<ActorDefsProfileAssociated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<serde_json::Value>,
    pub did: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<std::string::String>,
    pub handle: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ActorDefsStatusView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<ActorDefsVerificationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ActorDefsViewerState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsProfileViewDetailed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<ActorDefsProfileAssociated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<std::string::String>,
    pub did: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "followersCount")]
    pub followers_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "followsCount")]
    pub follows_count: Option<i64>,
    pub handle: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "indexedAt")]
    pub indexed_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "joinedViaStarterPack")]
    pub joined_via_starter_pack: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pinnedPost")]
    pub pinned_post: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "postsCount")]
    pub posts_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ActorDefsStatusView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<ActorDefsVerificationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ActorDefsViewerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsSavedFeed {
    pub id: std::string::String,
    pub pinned: bool,
    #[serde(rename = "type")]
    pub r#type: std::string::String,
    pub value: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsSavedFeedsPref {
    pub pinned: Vec<std::string::String>,
    pub saved: Vec<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "timelineIndex")]
    pub timeline_index: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsSavedFeedsPrefV2 {
    pub items: Vec<ActorDefsSavedFeed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsStatusView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isDisabled")]
    pub is_disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    pub record: serde_json::Value,
    pub status: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsThreadViewPref {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsVerificationPrefs {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hideBadges")]
    pub hide_badges: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsVerificationState {
    #[serde(rename = "trustedVerifierStatus")]
    pub trusted_verifier_status: std::string::String,
    pub verifications: Vec<ActorDefsVerificationView>,
    #[serde(rename = "verifiedStatus")]
    pub verified_status: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsVerificationView {
    #[serde(rename = "createdAt")]
    pub created_at: std::string::String,
    #[serde(rename = "isValid")]
    pub is_valid: bool,
    pub issuer: std::string::String,
    pub uri: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDefsViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "activitySubscription")]
    pub activity_subscription: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blockedBy")]
    pub blocked_by: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blockingByList")]
    pub blocking_by_list: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "followedBy")]
    pub followed_by: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "knownFollowers")]
    pub known_followers: Option<ActorDefsKnownFollowers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mutedByList")]
    pub muted_by_list: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsBlockedAuthor {
    pub did: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ActorDefsViewerState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsBlockedPost {
    pub author: FeedDefsBlockedAuthor,
    pub blocked: bool,
    pub uri: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsFeedViewPost {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "feedContext")]
    pub feed_context: Option<std::string::String>,
    pub post: FeedDefsPostView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<FeedDefsReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "reqId")]
    pub req_id: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsGeneratorView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "acceptsInteractions")]
    pub accepts_interactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<std::string::String>,
    pub cid: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentMode")]
    pub content_mode: Option<std::string::String>,
    pub creator: ActorDefsProfileView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "descriptionFacets")]
    pub description_facets: Option<Vec<serde_json::Value>>,
    pub did: std::string::String,
    #[serde(rename = "displayName")]
    pub display_name: std::string::String,
    #[serde(rename = "indexedAt")]
    pub indexed_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "likeCount")]
    pub like_count: Option<i64>,
    pub uri: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<FeedDefsGeneratorViewerState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsGeneratorViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsInteraction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "feedContext")]
    pub feed_context: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "reqId")]
    pub req_id: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsNotFoundPost {
    #[serde(rename = "notFound")]
    pub not_found: bool,
    pub uri: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsPostView {
    pub author: ActorDefsProfileViewBasic,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bookmarkCount")]
    pub bookmark_count: Option<i64>,
    pub cid: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    #[serde(rename = "indexedAt")]
    pub indexed_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "likeCount")]
    pub like_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "quoteCount")]
    pub quote_count: Option<i64>,
    pub record: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "replyCount")]
    pub reply_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "repostCount")]
    pub repost_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<FeedDefsThreadgateView>,
    pub uri: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<FeedDefsViewerState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsReasonPin {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsReasonRepost {
    pub by: ActorDefsProfileViewBasic,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<std::string::String>,
    #[serde(rename = "indexedAt")]
    pub indexed_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsReplyRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "grandparentAuthor")]
    pub grandparent_author: Option<ActorDefsProfileViewBasic>,
    pub parent: serde_json::Value,
    pub root: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsSkeletonFeedPost {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "feedContext")]
    pub feed_context: Option<std::string::String>,
    pub post: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsSkeletonReasonPin {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsSkeletonReasonRepost {
    pub repost: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsThreadContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rootAuthorLike")]
    pub root_author_like: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsThreadViewPost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<serde_json::Value>,
    pub post: FeedDefsPostView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "threadContext")]
    pub thread_context: Option<FeedDefsThreadContext>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsThreadgateView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedDefsViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmarked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "embeddingDisabled")]
    pub embedding_disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "replyDisabled")]
    pub reply_disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "threadMuted")]
    pub thread_muted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthorFeedParams {
    pub actor: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "includePins")]
    pub include_pins: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthorFeedOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<std::string::String>,
    pub feed: Vec<FeedDefsFeedViewPost>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEntity {
    pub index: PostTextSlice,
    #[serde(rename = "type")]
    pub r#type: std::string::String,
    pub value: std::string::String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    #[serde(rename = "$type", default = "default_post_type")]
    pub r#type: std::string::String,
    #[serde(rename = "createdAt")]
    pub created_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<PostEntity>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<PostReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<std::string::String>>,
    pub text: std::string::String,
}

fn default_post_type() -> std::string::String {
    "app.bsky.feed.post".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostReplyRef {
    pub parent: serde_json::Value,
    pub root: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostTextSlice {
    pub end: i64,
    pub start: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFollowersParams {
    pub actor: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFollowersOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<std::string::String>,
    pub followers: Vec<ActorDefsProfileView>,
    pub subject: ActorDefsProfileView,
}
