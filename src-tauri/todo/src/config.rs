use anyhow::Context;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "mynd";

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum SaveFileFormat {
    Json,
    Binary,
}

#[derive(Serialize, Deserialize)]
pub struct MyndConfig {
    pub save_file_format: SaveFileFormat,
}

impl Default for MyndConfig {
    fn default() -> Self {
        Self {
            save_file_format: SaveFileFormat::Binary,
        }
    }
}

pub fn load_config() -> anyhow::Result<MyndConfig> {
    confy::load::<MyndConfig>(APP_NAME, None).context("failed to load cli configs")
}

pub fn store_config(cfg: MyndConfig) -> anyhow::Result<()> {
    confy::store(APP_NAME, None, cfg).context("failed to store cli configs")
}
