use std::{env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    #[serde(default)]
    pub identity: IdentityConfig,
    #[serde(default)]
    pub services: ServiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IdentityConfig {
    pub identifier: String,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self { identifier: "did:plc:xg2vq45muivyy3xwatcehspu".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceConfig {
    pub public_api_base: String,
    pub plc_directory_base: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            public_api_base: "https://public.api.bsky.app".to_string(),
            plc_directory_base: "https://plc.directory".to_string(),
        }
    }
}

impl AppConfig {
    pub const FIELD_NAMES: [&'static str; 3] = [
        "identity.identifier",
        "services.public-api-base",
        "services.plc-directory-base",
    ];

    pub fn load(path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let Some(path) = resolve_config_path(path) else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Read { path: path.clone(), source })?;

        toml::from_str(&contents).map_err(|source| ConfigError::Parse { path, source })
    }

    pub fn save(&self, path: Option<PathBuf>) -> Result<PathBuf, ConfigError> {
        let path = resolve_config_path(path).ok_or(ConfigError::MissingPath)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| ConfigError::CreateDir { path: parent.to_path_buf(), source })?;
        }

        let contents = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;

        fs::write(&path, contents).map_err(|source| ConfigError::Write { path: path.clone(), source })?;

        Ok(path)
    }

    pub fn set_field(&mut self, field: &str, value: String) -> Result<(), ConfigError> {
        match field {
            "identity.identifier" => self.identity.identifier = value,
            "services.public-api-base" => self.services.public_api_base = value,
            "services.plc-directory-base" => self.services.plc_directory_base = value,
            _ => return Err(ConfigError::UnknownField(field.to_string())),
        }

        Ok(())
    }

    pub fn get_field(&self, field: &str) -> Result<&str, ConfigError> {
        match field {
            "identity.identifier" => Ok(&self.identity.identifier),
            "services.public-api-base" => Ok(&self.services.public_api_base),
            "services.plc-directory-base" => Ok(&self.services.plc_directory_base),
            _ => Err(ConfigError::UnknownField(field.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not determine config path; pass --config PATH or set XDG_CONFIG_HOME/HOME")]
    MissingPath,
    #[error("failed to read config at {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to create config directory at {path}: {source}")]
    CreateDir { path: PathBuf, source: std::io::Error },
    #[error("failed to write config at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error("failed to parse config at {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("failed to serialize config: {0}")]
    Serialize(toml::ser::Error),
    #[error("unknown config field {0:?}")]
    UnknownField(String),
}

fn resolve_config_path(path: Option<PathBuf>) -> Option<PathBuf> {
    path.or_else(default_config_path)
}

fn default_config_path() -> Option<PathBuf> {
    let config_home = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let base = config_home.or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("atproto-tools").join("config.toml"))
}
