use atp_tools_core::{ClientError, ParallelTaskError};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FollowsReportError {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error("failed to fetch follow posts in parallel: {0}")]
    ParallelFetch(ParallelTaskError<ClientError>),
    #[error("could not determine cache directory; set XDG_CACHE_HOME or HOME")]
    MissingCacheDir,
    #[error("failed to create cache directory at {path}: {source}")]
    CreateCacheDir { path: PathBuf, source: std::io::Error },
    #[error("failed to read cache at {path}: {source}")]
    ReadCache { path: PathBuf, source: std::io::Error },
    #[error("failed to parse cache at {path}: {source}")]
    ParseCache { path: PathBuf, source: serde_json::Error },
    #[error("failed to serialize cache: {0}")]
    SerializeCache(serde_json::Error),
    #[error("failed to write cache at {path}: {source}")]
    WriteCache { path: PathBuf, source: std::io::Error },
}
