use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct LexiconSyncSpec {
    pub repo: String,
    pub commit: String,
    pub source_path: String,
    pub dest_dir: PathBuf,
    pub files: Vec<String>,
    pub preserve_paths: bool,
}

#[derive(Debug, Clone)]
pub struct LexiconSyncReport {
    pub commit: String,
    pub written: Vec<PathBuf>,
}

pub async fn sync_lexicons(spec: LexiconSyncSpec) -> Result<LexiconSyncReport, LexiconSyncError> {
    std::fs::create_dir_all(&spec.dest_dir)
        .map_err(|source| LexiconSyncError::CreateDir { path: spec.dest_dir.clone(), source })?;

    let tmp =
        tempfile::tempdir().map_err(|source| LexiconSyncError::CreateDir { path: std::env::temp_dir(), source })?;

    run_git(
        [
            "clone",
            "--quiet",
            &clone_url(&spec.repo),
            tmp.path().to_string_lossy().as_ref(),
        ],
        None,
    )?;
    run_git(["checkout", "--quiet", &spec.commit], Some(tmp.path()))?;

    let mut written = Vec::with_capacity(spec.files.len());

    for file in &spec.files {
        let source = tmp.path().join(&spec.source_path).join(file);
        let body = std::fs::read_to_string(&source)
            .map_err(|error| LexiconSyncError::Read { path: source.clone(), source: error })?;
        let json = serde_json::from_str::<serde_json::Value>(&body)
            .map_err(|source| LexiconSyncError::Json { file: file.clone(), source })?;
        let pretty = serde_json::to_string_pretty(&json)
            .map_err(|source| LexiconSyncError::Json { file: file.clone(), source })?;
        let dest = if spec.preserve_paths {
            spec.dest_dir.join(clean_relative_path(file)?)
        } else {
            spec.dest_dir.join(dest_file_name(file)?)
        };

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|source| LexiconSyncError::CreateDir { path: parent.to_path_buf(), source })?;
        }

        std::fs::write(&dest, format!("{pretty}\n"))
            .map_err(|source| LexiconSyncError::Write { path: dest.clone(), source })?;
        written.push(dest);
    }

    Ok(LexiconSyncReport { commit: spec.commit, written })
}

#[derive(Debug, thiserror::Error)]
pub enum LexiconSyncError {
    #[error("failed to create lexicon directory at {path}: {source}")]
    CreateDir { path: PathBuf, source: std::io::Error },
    #[error("failed to read lexicon at {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to parse fetched lexicon JSON for {file}: {source}")]
    Json { file: String, source: serde_json::Error },
    #[error("invalid lexicon filename {0:?}")]
    InvalidFileName(String),
    #[error("failed to write lexicon at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error("git command failed: {0}")]
    Git(String),
}

fn clone_url(repo: &str) -> String {
    if repo.starts_with("http://") || repo.starts_with("https://") || repo.starts_with("ssh://") {
        repo.to_string()
    } else if repo.matches('/').count() == 1 {
        format!("https://github.com/{repo}.git")
    } else {
        format!("https://{repo}")
    }
}

fn run_git<const N: usize>(args: [&str; N], current_dir: Option<&Path>) -> Result<(), LexiconSyncError> {
    let mut command = Command::new("git");
    command.args(args);

    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }

    let output = command
        .output()
        .map_err(|source| LexiconSyncError::Git(source.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = if stdout.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        format!("{}\n{}", stderr.trim(), stdout.trim())
    };

    Err(LexiconSyncError::Git(message))
}

fn dest_file_name(file: &str) -> Result<&Path, LexiconSyncError> {
    Path::new(file)
        .file_name()
        .map(Path::new)
        .ok_or_else(|| LexiconSyncError::InvalidFileName(file.to_string()))
}

fn clean_relative_path(file: &str) -> Result<&Path, LexiconSyncError> {
    let path = Path::new(file);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(LexiconSyncError::InvalidFileName(file.to_string()));
    }

    Ok(path)
}
