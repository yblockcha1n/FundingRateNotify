use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub symbols: Vec<String>,
    pub notification_times: Vec<u32>,
    pub debug_push: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/config.toml");
        let contents = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}