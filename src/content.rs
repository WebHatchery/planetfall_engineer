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
    pub title: String,
    pub fabrication_start_fu: u32,
    pub reference_tick_min: u64,
    pub reference_tick_max: u64,
    pub source_rate_vu: u32,
    pub surge_start_tick: Option<u64>,
    pub surge_end_tick: Option<u64>,
    pub surge_rate_vu: u32,
    pub objective_min_vu: u32,
    pub objective_max_vu: u32,
    pub stability_ticks: u32,
    pub failure_ticks: u32,
    pub advisory_tick: u64,
    pub warning_tick: u64,
    pub hazard_threshold_vu: u32,
    pub hazard_reason: String,
    pub deposits: Vec<DepositRecord>,
    pub power_sources: Vec<PowerSourceRecord>,
    pub briefing: String,
    pub field_guide: String,
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
    pub prompts: Vec<TutorialPromptRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TutorialPromptRecord {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub action: String,
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

    pub fn tutorial_prompt(&self, step_id: &str) -> Option<&TutorialPromptRecord> {
        self.tutorials
            .iter()
            .flat_map(|tutorial| tutorial.prompts.iter())
            .find(|prompt| prompt.id == step_id)
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
    for fluid in &registry.fluids {
        validate_id("fluid", &fluid.id, errors);
    }
    for device in &registry.devices {
        validate_id("device", &device.id, errors);
    }
    for mission in &registry.missions {
        validate_id("mission", &mission.id, errors);
    }
    for tutorial in &registry.tutorials {
        validate_id("tutorial", &tutorial.id, errors);
    }
    for map in &registry.maps {
        validate_id("map", &map.id, errors);
    }
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
        for deposit in &mission.deposits {
            validate_id(
                &format!("mission {} deposit", mission.id),
                &deposit.id,
                errors,
            );
        }
        for source in &mission.power_sources {
            validate_id(
                &format!("mission {} power source", mission.id),
                &source.id,
                errors,
            );
        }
    }
    for tutorial in &registry.tutorials {
        validate_unique(
            &format!("tutorial {} prompt", tutorial.id),
            tutorial.prompts.iter().map(|prompt| prompt.id.as_str()),
            errors,
        );
        for prompt in &tutorial.prompts {
            validate_id(
                &format!("tutorial {} prompt", tutorial.id),
                &prompt.id,
                errors,
            );
        }
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
        if registry
            .map(device.showcase_map_id.as_str())
            .is_some_and(|map| map.kind != "device_showcase")
        {
            errors.push(format!(
                "device {}: showcase map {} is not a device_showcase",
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
        } else {
            validate_asset_id(&format!("device {}", device.id), &device.asset_id, errors);
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
        validate_mission(registry, maps, mission, errors);
    }
}

fn validate_mission(
    registry: &ContentRegistry,
    maps: &HashSet<&str>,
    mission: &MissionRecord,
    errors: &mut Vec<String>,
) {
    if !maps.contains(mission.map_id.as_str()) {
        errors.push(format!(
            "mission {}: missing map {}",
            mission.id, mission.map_id
        ));
    }
    let map = registry.map(mission.map_id.as_str());
    if map.is_some_and(|map| map.kind != "campaign") {
        errors.push(format!(
            "mission {}: map {} is not a campaign map",
            mission.id, mission.map_id
        ));
    }
    validate_mission_rules(mission, errors);
    validate_mission_copy(mission, errors);
    validate_mission_assets(mission, map, errors);
}

fn validate_mission_rules(mission: &MissionRecord, errors: &mut Vec<String>) {
    if mission.title.trim().is_empty() {
        errors.push(format!("mission {}: title must not be empty", mission.id));
    }
    if mission.reference_tick_min >= mission.reference_tick_max {
        errors.push(format!(
            "mission {}: reference tick range must be ascending",
            mission.id
        ));
    }
    if mission.source_rate_vu == 0 {
        errors.push(format!(
            "mission {}: source rate must be positive",
            mission.id
        ));
    }
    match (mission.surge_start_tick, mission.surge_end_tick) {
        (Some(start), Some(end)) if start < end && mission.surge_rate_vu > mission.source_rate_vu => {}
        (None, None) if mission.surge_rate_vu == 0 => {}
        _ => errors.push(format!(
            "mission {}: surge schedule must have an ascending window and a higher rate, or be disabled",
            mission.id
        )),
    }
    if mission.objective_min_vu == 0
        || mission.objective_min_vu > mission.objective_max_vu
        || mission.stability_ticks == 0
        || mission.failure_ticks == 0
    {
        errors.push(format!(
            "mission {}: objective and stability bounds are invalid",
            mission.id
        ));
    }
    if mission.fabrication_start_fu > 10_000 {
        errors.push(format!(
            "mission {}: starting fabrication exceeds slice bound",
            mission.id
        ));
    }
}

fn validate_mission_copy(mission: &MissionRecord, errors: &mut Vec<String>) {
    if mission.hazard_reason.trim().is_empty() {
        errors.push(format!(
            "mission {}: hazard_reason must not be empty",
            mission.id
        ));
    }
    if mission.field_guide.trim().is_empty() {
        errors.push(format!(
            "mission {}: field_guide must not be empty",
            mission.id
        ));
    }
    if mission.briefing.trim().is_empty() || mission.objective.trim().is_empty() {
        errors.push(format!(
            "mission {}: briefing and objective are required",
            mission.id
        ));
    }
}

fn validate_mission_assets(
    mission: &MissionRecord,
    map: Option<&MapRecord>,
    errors: &mut Vec<String>,
) {
    for deposit in &mission.deposits {
        if let Some(map) = map {
            validate_position(
                &format!("mission {} deposit {}", mission.id, deposit.id),
                deposit.position,
                map,
                errors,
            );
        }
        if deposit.yield_fu == 0 {
            errors.push(format!(
                "mission {} deposit {}: yield must be positive",
                mission.id, deposit.id
            ));
        }
        validate_named_asset(
            &format!("mission {} deposit {}", mission.id, deposit.id),
            deposit.asset_id.as_str(),
            errors,
        );
    }
    for source in &mission.power_sources {
        if let Some(map) = map {
            validate_position(
                &format!("mission {} power source {}", mission.id, source.id),
                source.position,
                map,
                errors,
            );
        }
        if source.output_eu_per_tick == 0 {
            errors.push(format!(
                "mission {} power source {}: output must be positive",
                mission.id, source.id
            ));
        }
        validate_named_asset(
            &format!("mission {} power source {}", mission.id, source.id),
            source.asset_id.as_str(),
            errors,
        );
    }
}

fn validate_named_asset(label: &str, asset_id: &str, errors: &mut Vec<String>) {
    if asset_id.trim().is_empty() {
        errors.push(format!("{label}: asset_id must not be empty"));
    } else {
        validate_asset_id(label, asset_id, errors);
    }
}
fn validate_tutorials(registry: &ContentRegistry, errors: &mut Vec<String>) {
    for tutorial in &registry.tutorials {
        let namespace = tutorial.id.strip_suffix("_power").unwrap_or(&tutorial.id);
        if tutorial.steps.is_empty()
            || tutorial
                .steps
                .iter()
                .any(|step| !step.starts_with(namespace))
        {
            errors.push(format!(
                "tutorial {}: expected non-empty namespaced steps",
                tutorial.id,
            ));
        }
        let step_ids: HashSet<&str> = tutorial.steps.iter().map(String::as_str).collect();
        for prompt in &tutorial.prompts {
            if !step_ids.contains(prompt.id.as_str()) {
                errors.push(format!(
                    "tutorial {}: prompt {} is not present in steps",
                    tutorial.id, prompt.id
                ));
            }
            if prompt.title.trim().is_empty()
                || prompt.instruction.trim().is_empty()
                || prompt.action.trim().is_empty()
            {
                errors.push(format!(
                    "tutorial {} prompt {}: player-facing text is required",
                    tutorial.id, prompt.id
                ));
            }
        }
        if tutorial.prompts.len() != tutorial.steps.len() {
            errors.push(format!(
                "tutorial {}: every step needs exactly one prompt",
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

fn validate_id(kind: &str, id: &str, errors: &mut Vec<String>) {
    if id.is_empty()
        || id.chars().any(|character| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        })
    {
        errors.push(format!("{kind} {id}: id must use lowercase snake_case"));
    }
}

fn validate_position(context: &str, position: [u16; 2], map: &MapRecord, errors: &mut Vec<String>) {
    if position[0] >= map.width || position[1] >= map.height {
        errors.push(format!(
            "{context}: position [{}, {}] is outside {}×{} map {}",
            position[0], position[1], map.width, map.height, map.id
        ));
    }
}

fn validate_asset_id(context: &str, asset_id: &str, errors: &mut Vec<String>) {
    if !matches!(
        asset_id.split('_').next(),
        Some("placeholder" | "deposit" | "power")
    ) {
        errors.push(format!(
            "{context}: asset_id {asset_id} is not a registered or approved procedural asset"
        ));
    }
}
