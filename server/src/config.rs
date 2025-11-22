use crate::error::AppError;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub port: u16,
    pub root_dir: String,
    pub session_timeout_minutes: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeatureConfig {
    pub enable_hdf5: bool,
    pub enable_watch: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub features: FeatureConfig,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&content)?;
        Ok(cfg)
    }
}
