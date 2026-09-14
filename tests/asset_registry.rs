//! Toolkit-backed packaging manifest coverage.

use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::Deserialize;
use std::collections::BTreeSet;

const ASSET_REGISTRY_JSON: &str = macroquad_toolkit::include_json_str!("../asset_registry.json");
const TEXTURE_MANIFEST_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");

#[derive(Debug, Deserialize)]
struct AssetRegistry {
    version: u32,
    assets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TextureRecord {
    path: String,
}

#[test]
fn asset_registry_matches_the_runtime_texture_manifest() {
    let registry: AssetRegistry =
        load_embedded_json_labeled("asset_registry", ASSET_REGISTRY_JSON).expect("valid registry");
    assert_eq!(registry.version, 1);
    let registered: BTreeSet<&str> = registry.assets.iter().map(String::as_str).collect();

    let texture_manifest: Vec<TextureRecord> =
        load_embedded_json_labeled("texture_manifest", TEXTURE_MANIFEST_JSON)
            .expect("valid texture manifest");
    let runtime_textures: BTreeSet<&str> = texture_manifest
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();

    assert_eq!(registered, runtime_textures);
    assert!(
        registered.is_empty(),
        "Planetfall Engineer renders art procedurally"
    );
}
