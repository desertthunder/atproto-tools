use std::{
    fs,
    path::{Path, PathBuf},
};

use reqwest::Url;

#[derive(Debug, Clone)]
pub struct LexiconSyncSpec {
    pub repo: String,
    pub commit: String,
    pub source_path: String,
    pub dest_dir: PathBuf,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LexiconSyncReport {
    pub commit: String,
    pub written: Vec<PathBuf>,
}

pub async fn sync_lexicons(spec: LexiconSyncSpec) -> Result<LexiconSyncReport, LexiconSyncError> {
    fs::create_dir_all(&spec.dest_dir)
        .map_err(|source| LexiconSyncError::CreateDir { path: spec.dest_dir.clone(), source })?;

    let http = reqwest::Client::new();
    let mut written = Vec::with_capacity(spec.files.len());

    for file in &spec.files {
        let source_url = raw_github_url(&spec.repo, &spec.commit, &spec.source_path, file)?;
        let body = http.get(source_url).send().await?.error_for_status()?.text().await?;
        let json = serde_json::from_str::<serde_json::Value>(&body)
            .map_err(|source| LexiconSyncError::Json { file: file.clone(), source })?;
        let pretty = serde_json::to_string_pretty(&json)
            .map_err(|source| LexiconSyncError::Json { file: file.clone(), source })?;

        let dest = spec.dest_dir.join(file);
        fs::write(&dest, format!("{pretty}\n"))
            .map_err(|source| LexiconSyncError::Write { path: dest.clone(), source })?;
        written.push(dest);
    }

    Ok(LexiconSyncReport { commit: spec.commit, written })
}

#[derive(Debug, thiserror::Error)]
pub enum LexiconSyncError {
    #[error("invalid raw GitHub URL for {file}: {source}")]
    Url { file: String, source: url::ParseError },
    #[error("failed to create lexicon directory at {path}: {source}")]
    CreateDir { path: PathBuf, source: std::io::Error },
    #[error("failed to parse fetched lexicon JSON for {file}: {source}")]
    Json { file: String, source: serde_json::Error },
    #[error("failed to write lexicon at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

fn raw_github_url(repo: &str, commit: &str, source_path: &str, file: &str) -> Result<Url, LexiconSyncError> {
    let path = Path::new(source_path).join(file);
    let value = format!(
        "https://raw.githubusercontent.com/{repo}/{commit}/{}",
        path.to_string_lossy().replace('\\', "/")
    );

    Url::parse(&value).map_err(|source| LexiconSyncError::Url { file: file.to_string(), source })
}
