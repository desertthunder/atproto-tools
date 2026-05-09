use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorRepoInfo {
    pub profile: ActorProfileDetailed,
    pub repo: RepoDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorProfileDetailed {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub pronouns: Option<String>,
    pub website: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub followers_count: Option<u64>,
    pub follows_count: Option<u64>,
    pub posts_count: Option<u64>,
    pub associated: Option<Value>,
    pub joined_via_starter_pack: Option<Value>,
    pub indexed_at: Option<String>,
    pub created_at: Option<String>,
    pub viewer: Option<Value>,
    #[serde(default)]
    pub labels: Vec<Value>,
    pub pinned_post: Option<Value>,
    pub verification: Option<Value>,
    pub status: Option<Value>,
    pub debug: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoDescription {
    pub handle: String,
    pub did: String,
    pub did_doc: Value,
    pub collections: Vec<String>,
    pub handle_is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    #[serde(default)]
    pub service: Vec<DidDocumentService>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocumentService {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub service_endpoint: String,
}
