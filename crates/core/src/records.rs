use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRecordsResponse<T> {
    pub records: Vec<RepoRecord<T>>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoRecord<T> {
    pub uri: String,
    pub cid: String,
    pub value: T,
}
