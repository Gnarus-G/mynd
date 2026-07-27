use anyhow::Context;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "mynd";

#[derive(ValueEnum, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SaveFileFormat {
    Json,
    Binary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyndConfig {
    pub save_file_format: SaveFileFormat,
    #[serde(default)]
    pub web_url: Option<String>,
}

impl Default for MyndConfig {
    fn default() -> Self {
        Self {
            save_file_format: SaveFileFormat::Binary,
            web_url: None,
        }
    }
}

pub fn load_config() -> anyhow::Result<MyndConfig> {
    confy::load::<MyndConfig>(APP_NAME, None).context("failed to load cli configs")
}

pub fn store_config(cfg: MyndConfig) -> anyhow::Result<()> {
    confy::store(APP_NAME, None, cfg).context("failed to store cli configs")
}

pub fn web_url() -> anyhow::Result<String> {
    let url = load_config()?
        .web_url
        .context("Mynd web URL is not configured; run `todo config set --web-url <URL>`")?;
    validate_web_url(&url)?;
    Ok(url)
}

pub fn validate_web_url(value: &str) -> anyhow::Result<()> {
    let url = url::Url::parse(value).context("Mynd web URL is invalid")?;
    let loopback_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !loopback_http {
        anyhow::bail!("Mynd web URL must use HTTPS or a loopback HTTP address");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_config_without_web_url_still_loads() {
        let config: MyndConfig = serde_json::from_str(r#"{"save_file_format":"Binary"}"#).unwrap();

        assert!(config.web_url.is_none());
    }
}
