//! Embedded slice registry and cross-reference validation.

use serde::Deserialize;
use std::collections::HashSet;

const REGISTRY_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/content_registry.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentRegistry {
    pub schema_version: u32,
    pub fluids: Vec<FluidRecord>,
    pub devices: Vec<DeviceRecord>,
    pub missions: Vec<MissionRecord>,
    pub tutorials: Vec<TutorialRecord>,
    pub maps: Vec<MapRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FluidRecord {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceRecord {
    pub id: String,
    pub showcase_map_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionRecord {
    pub id: String,
    pub map_id: String,
    pub sequence: u8,
    pub budget_credits: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TutorialRecord {
    pub id: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapRecord {
    pub id: String,
    pub kind: String,
    pub width: u16,
    pub height: u16,
}

impl ContentRegistry {
    pub fn load() -> Result<Self, String> {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/data/content_registry.json",
            REGISTRY_JSON,
        )
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema_version != 1 {
            errors.push(format!(
                "schema_version: expected 1, got {}",
                self.schema_version
            ));
        }
        validate_unique(
            "fluid",
            self.fluids.iter().map(|record| record.id.as_str()),
            &mut errors,
        );
        validate_unique(
            "device",
            self.devices.iter().map(|record| record.id.as_str()),
            &mut errors,
        );
        validate_unique(
            "mission",
            self.missions.iter().map(|record| record.id.as_str()),
            &mut errors,
        );
        validate_unique(
            "tutorial",
            self.tutorials.iter().map(|record| record.id.as_str()),
            &mut errors,
        );
        validate_unique(
            "map",
            self.maps.iter().map(|record| record.id.as_str()),
            &mut errors,
        );
        let maps: HashSet<&str> = self.maps.iter().map(|record| record.id.as_str()).collect();
        let enabled: HashSet<&str> = self
            .fluids
            .iter()
            .filter(|record| record.enabled)
            .map(|record| record.id.as_str())
            .collect();
        if enabled != HashSet::from(["water", "lava", "steam", "toxic_slurry"]) {
            errors.push(
                "fluids: enabled slice set does not match the four supported materials".into(),
            );
        }
        for device in &self.devices {
            if !maps.contains(device.showcase_map_id.as_str()) {
                errors.push(format!(
                    "device {}: missing showcase map {}",
                    device.id, device.showcase_map_id
                ));
            }
        }
        for mission in &self.missions {
            if !maps.contains(mission.map_id.as_str()) {
                errors.push(format!(
                    "mission {}: missing map {}",
                    mission.id, mission.map_id
                ));
            }
            if mission.budget_credits == 0 {
                errors.push(format!("mission {}: budget must be positive", mission.id));
            }
        }
        for tutorial in &self.tutorials {
            if tutorial.steps.len() != 12
                || tutorial
                    .steps
                    .iter()
                    .any(|step| !step.starts_with(&tutorial.id))
            {
                errors.push(format!(
                    "tutorial {}: expected twelve namespaced steps",
                    tutorial.id
                ));
            }
        }
        let sequences: HashSet<u8> = self
            .missions
            .iter()
            .map(|mission| mission.sequence)
            .collect();
        if sequences != HashSet::from([1, 2, 3]) {
            errors.push(format!(
                "missions.sequence: expected 1, 2, 3, got {sequences:?}"
            ));
        }
        for map in &self.maps {
            if map.width == 0 || map.height == 0 {
                errors.push(format!("map {}: dimensions must be positive", map.id));
            }
            if !matches!(
                map.kind.as_str(),
                "campaign" | "fluid_lab" | "device_showcase"
            ) {
                errors.push(format!("map {}: unknown kind {}", map.id, map.kind));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_unique<'a>(kind: &str, ids: impl Iterator<Item = &'a str>, errors: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(format!("{kind} {id}: duplicate id"));
        }
    }
}

#[cfg(test)]
mod tests;
