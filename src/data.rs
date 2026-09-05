//! Embedded, deterministic foundation content.

use crate::content::ContentRegistry;
use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::{Deserialize, Serialize};

const CONFIG_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const TEXTURES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");

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
    pub content: ContentRegistry,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", CONFIG_JSON)?;
        let texture_manifest = load_embedded_json_labeled("texture_manifest", TEXTURES_JSON)?;
        let content = ContentRegistry::load()?;
        content
            .validate()
            .map_err(|errors| format!("content validation failed:\n{}", errors.join("\n")))?;
        Ok(Self {
            config,
            texture_manifest,
            content,
        })
    }
}

#[cfg(test)]
mod tests;
