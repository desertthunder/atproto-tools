use super::generated::Note;
use atp_tools_core::{AtprotoClient, RepoRecord};
use comrak::{Arena, Options, format_commonmark, parse_document};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const NOTE_COLLECTION: &str = "at.margin.note";

#[derive(Debug, Clone)]
pub struct SourceNotesDocument {
    pub title: String,
    pub source: String,
    pub updated: String,
    pub sha: String,
    pub notes: Vec<RepoRecord<Note>>,
}

pub async fn export_notes(client: &AtprotoClient, actor: &str) -> Result<Vec<SourceNotesDocument>, MarginExportError> {
    let notes = client.list_records::<Note>(actor, NOTE_COLLECTION).await?;
    Ok(group_notes_by_source(notes))
}

pub async fn export_source_notes(
    client: &AtprotoClient, actor: &str, source: &str,
) -> Result<Option<SourceNotesDocument>, MarginExportError> {
    Ok(export_notes(client, actor)
        .await?
        .into_iter()
        .find(|document| document.source == source))
}

pub fn group_notes_by_source(notes: Vec<RepoRecord<Note>>) -> Vec<SourceNotesDocument> {
    let mut groups = BTreeMap::<String, Vec<RepoRecord<Note>>>::new();

    for note in notes {
        groups.entry(note.value.target.source.clone()).or_default().push(note);
    }

    groups
        .into_iter()
        .map(|(source, notes)| SourceNotesDocument::from_notes(&source, notes))
        .collect()
}

impl SourceNotesDocument {
    pub fn from_notes(source: &str, mut notes: Vec<RepoRecord<Note>>) -> Self {
        notes.sort_by(|left, right| left.value.created_at.cmp(&right.value.created_at));

        let title = notes
            .iter()
            .find_map(|record| record.value.target.title.clone())
            .unwrap_or_else(|| source.to_string());
        let updated = notes
            .iter()
            .filter_map(|record| record.value.modified_at.as_ref().or(Some(&record.value.created_at)))
            .max()
            .cloned()
            .unwrap_or_default();

        let sha = notes_sha(source, &notes);

        Self { title, source: source.to_string(), updated, sha, notes }
    }

    pub fn to_markdown(&self) -> Result<String, MarginExportError> {
        let frontmatter = toml::to_string(&Frontmatter {
            title: &self.title,
            source: &self.source,
            updated: &self.updated,
            sha: &self.sha,
        })?;
        let mut markdown = format!("+++\n{frontmatter}+++\n\n");

        for (index, note) in self.notes.iter().enumerate() {
            if index > 0 {
                markdown.push_str("---\n\n");
            }
            append_note(&mut markdown, note)?;
        }

        Ok(markdown)
    }

    pub fn filename(&self) -> String {
        format!(
            "{}.md",
            slugify(self.title.trim())
                .or_else(|| slugify(&self.source))
                .unwrap_or_else(|| "notes".to_string())
        )
    }
}

#[derive(Debug, Serialize)]
struct Frontmatter<'a> {
    title: &'a str,
    source: &'a str,
    updated: &'a str,
    sha: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum MarginExportError {
    #[error(transparent)]
    Client(#[from] atp_tools_core::ClientError),
    #[error("failed to serialize markdown frontmatter: {0}")]
    Frontmatter(#[from] toml::ser::Error),
    #[error("failed to format markdown note body: {0}")]
    Markdown(std::fmt::Error),
}

fn slugify(value: &str) -> Option<String> {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    (!slug.is_empty()).then_some(slug)
}

fn notes_sha(source: &str, notes: &[RepoRecord<Note>]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(b"\n");

    for record in notes {
        hasher.update(record.uri.as_bytes());
        hasher.update(b"\n");
        hasher.update(record.cid.as_bytes());
        hasher.update(b"\n");
        hasher.update(record.value.created_at.as_bytes());
        hasher.update(b"\n");

        if let Some(modified_at) = &record.value.modified_at {
            hasher.update(modified_at.as_bytes());
        }
        hasher.update(b"\n");

        if let Some(selector) = &record.value.target.selector {
            if let Some(exact) = &selector.exact {
                hasher.update(exact.as_bytes());
            }
        }
        hasher.update(b"\n");

        if let Some(body) = &record.value.body {
            if let Some(value) = &body.value {
                hasher.update(value.as_bytes());
            }
        }
        hasher.update(b"\n---\n");
    }

    format!("{:x}", hasher.finalize())
}

fn append_note(markdown: &mut String, record: &RepoRecord<Note>) -> Result<(), MarginExportError> {
    markdown.push_str(&format!("- uri: {}\n", record.uri));
    markdown.push_str(&format!("- cid: {}\n", record.cid));
    markdown.push_str(&format!("- created: {}\n", record.value.created_at));

    if let Some(modified_at) = &record.value.modified_at {
        markdown.push_str(&format!("- modified: {modified_at}\n"));
    }

    if let Some(tags) = &record.value.tags {
        if !tags.is_empty() {
            markdown.push_str(&format!("- tags: {}\n", tags.join(", ")));
        }
    }

    markdown.push('\n');

    if let Some(selector) = &record.value.target.selector {
        if let Some(exact) = &selector.exact {
            markdown.push_str("> ");
            markdown.push_str(&exact.replace('\n', "\n> "));
            markdown.push_str("\n\n");
        }
    }

    if let Some(body) = &record.value.body {
        if let Some(value) = &body.value {
            markdown.push_str(&normalize_markdown(value)?);
            markdown.push('\n');
        }
    }

    Ok(())
}

fn normalize_markdown(value: &str) -> Result<String, MarginExportError> {
    let arena = Arena::new();
    let root = parse_document(&arena, value, &Options::default());
    let mut output = String::new();
    format_commonmark(root, &Options::default(), &mut output).map_err(MarginExportError::Markdown)?;
    Ok(output)
}
