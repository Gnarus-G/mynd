use serde::{Deserialize, Serialize};

const APP_NAME: &str = "mynd";

#[derive(Serialize, Deserialize)]
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

pub fn load_config() -> MyndConfig {
    confy::load::<MyndConfig>(APP_NAME, None).expect("failed to load cli configs")
}

pub fn store_config(cfg: MyndConfig) {
    confy::store(APP_NAME, None, cfg).expect("failed to store cli configs")
}
