// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    #[serde(rename = "$type", default = "default_issue_type")]
    pub r#type: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<std::string::String>,
    #[serde(rename = "createdAt")]
    pub created_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "repoDid")]
    pub repo_did: Option<std::string::String>,
    pub title: std::string::String,
}

fn default_issue_type() -> std::string::String {
    "sh.tangled.repo.issue".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    #[serde(rename = "$type", default = "default_repo_type")]
    pub r#type: std::string::String,
    #[serde(rename = "createdAt")]
    pub created_at: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<std::string::String>,
    pub knot: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<std::string::String>>,
    pub name: std::string::String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "repoDid")]
    pub repo_did: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spindle: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<std::string::String>,
}

fn default_repo_type() -> std::string::String {
    "sh.tangled.repo".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringRecord {
    #[serde(rename = "$type", default = "default_string_record_type")]
    pub r#type: std::string::String,
    pub contents: std::string::String,
    #[serde(rename = "createdAt")]
    pub created_at: std::string::String,
    pub description: std::string::String,
    pub filename: std::string::String,
}

fn default_string_record_type() -> std::string::String {
    "sh.tangled.string".to_string()
}
