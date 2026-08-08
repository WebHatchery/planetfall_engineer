//! Embedded, deterministic foundation content.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::load_embedded_json;
use serde::{Deserialize, Serialize};

const CONFIG_JSON: &str = include_str!("../assets/data/game_config.json");
const TEXTURES_JSON: &str = include_str!("../assets/data/texture_manifest.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    pub version: String,
    pub world_width: usize,
    pub world_height: usize,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json(CONFIG_JSON)?;
        let texture_manifest = load_embedded_json(TEXTURES_JSON)?;
        Ok(Self { config, texture_manifest })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_foundation_data_loads() {
        let data = GameData::load().unwrap();
        assert_eq!(data.config.world_width, 32);
        assert_eq!(data.config.world_height, 20);
    }
}
