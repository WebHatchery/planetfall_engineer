//! Embedded slice registry and cross-reference validation.

use serde::Deserialize;
use std::collections::HashSet;

const REGISTRY_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/content_registry.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentRegistry {
    pub schema_version: u32,
    pub content_version: String,
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
    pub fabrication_cost_fu: u32,
    pub power_demand_eu_per_tick: u32,
    pub power_class: Option<String>,
    pub asset_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionRecord {
    pub id: String,
    pub map_id: String,
    pub sequence: u8,
    pub fabrication_start_fu: u32,
    pub deposits: Vec<DepositRecord>,
    pub power_sources: Vec<PowerSourceRecord>,
    pub briefing: String,
    pub objective: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepositRecord {
    pub id: String,
    pub position: [u16; 2],
    pub yield_fu: u32,
    pub asset_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PowerSourceRecord {
    pub id: String,
    pub position: [u16; 2],
    pub output_eu_per_tick: u32,
    pub enabled: bool,
    pub asset_id: String,
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
        validate_schema(self, &mut errors);
        validate_ids(self, &mut errors);
        let maps: HashSet<&str> = self.maps.iter().map(|record| record.id.as_str()).collect();
        validate_fluids(self, &mut errors);
        validate_devices(self, &maps, &mut errors);
        validate_missions(self, &maps, &mut errors);
        validate_tutorials(self, &mut errors);
        validate_maps(self, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn mission(&self, id: &str) -> Option<&MissionRecord> {
        self.missions.iter().find(|mission| mission.id == id)
    }

    pub fn map(&self, id: &str) -> Option<&MapRecord> {
        self.maps.iter().find(|map| map.id == id)
    }
}

fn validate_schema(registry: &ContentRegistry, errors: &mut Vec<String>) {
    if registry.schema_version != 2 {
        errors.push(format!(
            "schema_version: expected 2, got {}",
            registry.schema_version
        ));
    }
    if registry.content_version.trim().is_empty() {
        errors.push("content_version: must not be empty".into());
    }
}

fn validate_ids(registry: &ContentRegistry, errors: &mut Vec<String>) {
    validate_unique(
        "fluid",
        registry.fluids.iter().map(|record| record.id.as_str()),
        errors,
    );
    validate_unique(
        "device",
        registry.devices.iter().map(|record| record.id.as_str()),
        errors,
    );
    validate_unique(
        "mission",
        registry.missions.iter().map(|record| record.id.as_str()),
        errors,
    );
    validate_unique(
        "tutorial",
        registry.tutorials.iter().map(|record| record.id.as_str()),
        errors,
    );
    validate_unique(
        "map",
        registry.maps.iter().map(|record| record.id.as_str()),
        errors,
    );
    for mission in &registry.missions {
        validate_unique(
            &format!("mission {} deposit", mission.id),
            mission.deposits.iter().map(|record| record.id.as_str()),
            errors,
        );
        validate_unique(
            &format!("mission {} power source", mission.id),
            mission
                .power_sources
                .iter()
                .map(|record| record.id.as_str()),
            errors,
        );
    }
}

fn validate_fluids(registry: &ContentRegistry, errors: &mut Vec<String>) {
    let enabled: HashSet<&str> = registry
        .fluids
        .iter()
        .filter(|record| record.enabled)
        .map(|record| record.id.as_str())
        .collect();
    if enabled != HashSet::from(["water", "lava", "steam", "toxic_slurry"]) {
        errors.push("fluids: enabled slice set does not match the four supported materials".into());
    }
}

fn validate_devices(registry: &ContentRegistry, maps: &HashSet<&str>, errors: &mut Vec<String>) {
    for device in &registry.devices {
        if !maps.contains(device.showcase_map_id.as_str()) {
            errors.push(format!(
                "device {}: missing showcase map {}",
                device.id, device.showcase_map_id
            ));
        }
        if device.fabrication_cost_fu == 0 {
            errors.push(format!(
                "device {}: fabrication cost must be positive",
                device.id
            ));
        }
        if device.power_demand_eu_per_tick > 0
            && !matches!(
                device.power_class.as_deref(),
                Some("safety" | "transport" | "process" | "interface")
            )
        {
            errors.push(format!(
                "device {}: powered device needs a valid power class",
                device.id
            ));
        }
        if device.asset_id.trim().is_empty() {
            errors.push(format!("device {}: asset_id must not be empty", device.id));
        }
    }
}

fn validate_missions(registry: &ContentRegistry, maps: &HashSet<&str>, errors: &mut Vec<String>) {
    let sequences: HashSet<u8> = registry
        .missions
        .iter()
        .map(|mission| mission.sequence)
        .collect();
    if sequences != HashSet::from([1, 2, 3]) {
        errors.push(format!(
            "missions.sequence: expected 1, 2, 3, got {sequences:?}"
        ));
    }
    for mission in &registry.missions {
        if !maps.contains(mission.map_id.as_str()) {
            errors.push(format!(
                "mission {}: missing map {}",
                mission.id, mission.map_id
            ));
        }
        if mission.fabrication_start_fu > 10_000 {
            errors.push(format!(
                "mission {}: starting fabrication exceeds slice bound",
                mission.id
            ));
        }
        for deposit in &mission.deposits {
            if deposit.yield_fu == 0 {
                errors.push(format!(
                    "mission {} deposit {}: yield must be positive",
                    mission.id, deposit.id
                ));
            }
            if deposit.asset_id.trim().is_empty() {
                errors.push(format!(
                    "mission {} deposit {}: asset_id must not be empty",
                    mission.id, deposit.id
                ));
            }
        }
        for source in &mission.power_sources {
            if source.output_eu_per_tick == 0 {
                errors.push(format!(
                    "mission {} power source {}: output must be positive",
                    mission.id, source.id
                ));
            }
        }
        if mission.briefing.trim().is_empty() || mission.objective.trim().is_empty() {
            errors.push(format!(
                "mission {}: briefing and objective are required",
                mission.id
            ));
        }
    }
}

fn validate_tutorials(registry: &ContentRegistry, errors: &mut Vec<String>) {
    for tutorial in &registry.tutorials {
        let expected = if tutorial.id == "tutorial_l01" { 14 } else { 5 };
        let namespace = tutorial.id.strip_suffix("_power").unwrap_or(&tutorial.id);
        if tutorial.steps.len() != expected
            || tutorial
                .steps
                .iter()
                .any(|step| !step.starts_with(namespace))
        {
            errors.push(format!(
                "tutorial {}: expected {expected} namespaced steps",
                tutorial.id
            ));
        }
    }
}

fn validate_maps(registry: &ContentRegistry, errors: &mut Vec<String>) {
    for map in &registry.maps {
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
}

fn validate_unique<'a>(kind: &str, ids: impl Iterator<Item = &'a str>, errors: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(format!("{kind} {id}: duplicate id"));
        }
    }
}
