//! Deterministic campaign and failure-path replay harness.

use crate::{
    campaign::{apply_scheduled_events, load_campaign},
    mission::{Admission, CommandKind, MissionId, MissionPhase, MissionState},
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioKind {
    Reference,
    Alternate,
    Failure,
    InsufficientWaterRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioReport {
    pub id: MissionId,
    pub kind: ScenarioKind,
    pub initial_hash: u64,
    pub midpoint_hash: u64,
    pub final_hash: u64,
    pub ticks: u64,
    pub continuation_matches: bool,
    pub terminal_success: bool,
    pub terminal_failure: bool,
    pub objective_incomplete: bool,
    pub objective_progress: u32,
    pub mass_balance_error: i64,
    pub injected_vu: u64,
    pub drained_vu: u64,
    pub reacted_vu: u64,
    pub products_vu: u64,
    pub admitted_commands: u32,
    pub placed_devices: usize,
    pub reservoir_vu: u32,
    pub turbine_power: u64,
    pub relay_active: bool,
    pub relay_present: bool,
    pub relay_stored_vu: u32,
    pub relay_powered: bool,
    pub total_material_vu: u64,
    pub fabrication_available_fu: u32,
    pub fabrication_reserved_fu: u32,
    pub fabrication_spent_fu: u32,
    pub fabrication_refunded_fu: u32,
    pub fabrication_recovered_fu: u32,
    pub fabrication_balance_error: i64,
    pub power_authored_eu: u32,
    pub power_turbine_eu: u32,
    pub power_allocated_eu: u32,
    pub power_curtailed_eu: u32,
    pub power_deficit_eu: u32,
}

pub fn run_scenario(id: MissionId, kind: ScenarioKind) -> ScenarioReport {
    let mut map = load_campaign(id);
    let mut mission = MissionState::new(id);
    mission.start();
    let initial_hash = hash(&map.world, &mission);
    seed_scenario(&mut map.world, &mut mission, id, kind);
    for _ in 0..50 {
        tick_scenario(&mut map.world, &mut mission, id, kind);
    }
    let midpoint_hash = hash(&map.world, &mission);
    let (mut resumed_world, mut resumed_mission): (SimulationWorld, MissionState) =
        serde_json::to_vec(&(map.world.clone(), mission.clone()))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_else(|| (map.world.clone(), mission.clone()));
    while mission.phase == MissionPhase::Active && mission.tick < 2_200 {
        tick_scenario(&mut map.world, &mut mission, id, kind);
    }
    let final_hash = hash(&map.world, &mission);
    while resumed_mission.phase == MissionPhase::Active && resumed_mission.tick < 2_200 {
        tick_scenario(&mut resumed_world, &mut resumed_mission, id, kind);
    }
    let mass_balance_error = map.world.mass_balance_error();
    ScenarioReport {
        id,
        kind,
        initial_hash,
        midpoint_hash,
        final_hash,
        ticks: mission.tick,
        continuation_matches: final_hash == hash(&resumed_world, &resumed_mission),
        terminal_success: mission.phase == MissionPhase::Success,
        terminal_failure: mission.phase == MissionPhase::Failure,
        objective_incomplete: mission.objective_progress < objective_target(id),
        objective_progress: mission.objective_progress,
        mass_balance_error,
        injected_vu: map.world.ledger.injected,
        drained_vu: map.world.ledger.drained,
        reacted_vu: map.world.ledger.reacted,
        products_vu: map.world.ledger.products,
        admitted_commands: mission.command_count,
        placed_devices: map.world.devices.devices.len(),
        reservoir_vu: map
            .world
            .devices
            .devices
            .iter()
            .filter(|device| device.device == crate::devices::DeviceId::Reservoir)
            .map(|device| device.stored_vu)
            .sum(),
        turbine_power: map
            .world
            .devices
            .devices
            .iter()
            .filter(|device| device.device == crate::devices::DeviceId::FlowTurbine)
            .map(|device| device.cumulative_power)
            .sum(),
        relay_active: map
            .world
            .devices
            .devices
            .iter()
            .any(|device| device.device == crate::devices::DeviceId::RuneRelay && device.active),
        relay_present: map
            .world
            .devices
            .devices
            .iter()
            .any(|device| device.device == crate::devices::DeviceId::RuneRelay),
        relay_stored_vu: map
            .world
            .devices
            .devices
            .iter()
            .filter(|device| device.device == crate::devices::DeviceId::RuneRelay)
            .map(|device| device.stored_vu)
            .sum(),
        relay_powered: map
            .world
            .devices
            .devices
            .iter()
            .any(|device| device.device == crate::devices::DeviceId::RuneRelay && device.powered),
        total_material_vu: map.world.total_material_volume(),
        fabrication_available_fu: map.world.fabrication.available_fu,
        fabrication_reserved_fu: map.world.fabrication.reserved_fu,
        fabrication_spent_fu: map.world.fabrication.spent_fu,
        fabrication_refunded_fu: map.world.fabrication.refunded_fu,
        fabrication_recovered_fu: map.world.fabrication.recovered_fu,
        fabrication_balance_error: map.world.fabrication.balance_error(),
        power_authored_eu: map.world.power.authored_generation_eu,
        power_turbine_eu: map.world.power.turbine_generation_eu,
        power_allocated_eu: map.world.power.allocated_demand_eu,
        power_curtailed_eu: map.world.power.curtailed_supply_eu,
        power_deficit_eu: map.world.power.deficit_eu,
    }
}

fn seed_scenario(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
    match kind {
        // The replay represents the operator's opening recovery commands
        // explicitly, so the build below consumes finite authored stock.
        ScenarioKind::Reference | ScenarioKind::Alternate => recover_authored_stock(world),
        ScenarioKind::Failure => match id {
            MissionId::L01FirstFlow => {
                world.inject(CellPos { x: 24, y: 8 }, FluidId::Water, 4_000);
            }
            MissionId::L02HoldingLine => {
                world.inject(CellPos { x: 26, y: 16 }, FluidId::Water, 12_000);
            }
            MissionId::L03Firebreak => {
                world.inject(CellPos { x: 37, y: 15 }, FluidId::Lava, 4_000);
            }
        },
        ScenarioKind::InsufficientWaterRecovery => {
            if id == MissionId::L03Firebreak {
                let mut removed = 0u64;
                for pos in (8..=11).flat_map(|x| (3..=6).map(move |y| CellPos { x, y })) {
                    if let Some(index) = world.index(pos) {
                        removed += world.cells[index].surface_volume() as u64;
                        world.cells[index].surface.clear();
                    }
                }
                world.ledger.drained += removed;
                world.inject(CellPos { x: 11, y: 5 }, FluidId::Water, 500);
            }
        }
    }
    install_reference_build(world, mission, id, kind);
    if matches!(kind, ScenarioKind::Reference | ScenarioKind::Alternate) {
        world.set_sources_enabled(true);
    }
}

fn recover_authored_stock(world: &mut SimulationWorld) {
    let deposit_ids = world
        .deposits
        .iter()
        .map(|deposit| deposit.id.clone())
        .collect::<Vec<_>>();
    for id in deposit_ids {
        let _ = world.recover_deposit(&id);
    }
}

fn install_reference_build(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
    if matches!(
        kind,
        ScenarioKind::Failure | ScenarioKind::InsufficientWaterRecovery
    ) {
        return;
    }
    if mission.id == MissionId::L01FirstFlow {
        let _ = mission.skip_tutorial();
    }
    match id {
        MissionId::L01FirstFlow => install_l01_build(world, mission),
        MissionId::L02HoldingLine => install_l02_build(world, mission),
        MissionId::L03Firebreak => install_l03_build(world, mission),
    }
}

fn install_l01_build(world: &mut SimulationWorld, mission: &mut MissionState) {
    for position in [
        CellPos { x: 8, y: 10 },
        CellPos { x: 14, y: 10 },
        CellPos { x: 20, y: 10 },
    ] {
        let _ = mission.admit(CommandKind::SelectTerrain);
        let _ = world.terrain_edit(position, crate::simulation::TerrainAction::Raise);
    }
}

fn install_l02_build(world: &mut SimulationWorld, mission: &mut MissionState) {
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::Pump,
        CellPos { x: 5, y: 16 },
        0,
    );
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::FlowTurbine,
        CellPos { x: 27, y: 9 },
        0,
    );
    admit_many(
        world,
        mission,
        crate::devices::DeviceId::Pipe,
        [
            (9, 16),
            (11, 16),
            (13, 16),
            (15, 16),
            (19, 12),
            (20, 12),
            (20, 11),
            (20, 10),
            (23, 8),
            (25, 8),
            (27, 8),
            (29, 8),
        ],
    );
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::Reservoir,
        CellPos { x: 20, y: 8 },
        0,
    );
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::Spillway,
        CellPos { x: 39, y: 18 },
        0,
    );
    if let Some(entity_id) = world
        .devices
        .devices
        .iter()
        .find(|device| device.device == crate::devices::DeviceId::Reservoir)
        .map(|device| device.entity_id)
    {
        let _ = world.devices.set_enabled(entity_id, false);
    }
}

fn install_l03_build(world: &mut SimulationWorld, mission: &mut MissionState) {
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::Pump,
        CellPos { x: 11, y: 5 },
        0,
    );
    admit_many(
        world,
        mission,
        crate::devices::DeviceId::Pipe,
        [
            (15, 5),
            (17, 5),
            (19, 5),
            (20, 6),
            (20, 8),
            (20, 10),
            (20, 12),
            (20, 14),
            (22, 14),
            (23, 14),
            (24, 14),
            (25, 14),
            (26, 14),
            (27, 14),
            (27, 10),
            (27, 11),
            (27, 12),
            (27, 13),
            (28, 9),
            (29, 9),
            (30, 9),
            (31, 9),
            (32, 9),
            (33, 9),
            (34, 9),
            (35, 9),
            (36, 9),
            (37, 9),
            (37, 5),
            (37, 6),
            (37, 7),
            (37, 8),
        ],
    );
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::Floodgate,
        CellPos { x: 21, y: 14 },
        0,
    );
    if mission.admit(CommandKind::SetGate(2_500)) == Admission::Accepted {
        world
            .devices
            .set_selected_gate(CellPos { x: 21, y: 14 }, 2_500);
    }
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::FlowTurbine,
        CellPos { x: 27, y: 9 },
        0,
    );
    let _ = admit_build(
        world,
        mission,
        crate::devices::DeviceId::RuneRelay,
        CellPos { x: 38, y: 5 },
        0,
    );
}

fn admit_many(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    device: crate::devices::DeviceId,
    positions: impl IntoIterator<Item = (u16, u16)>,
) {
    for (x, y) in positions {
        let _ = admit_build(world, mission, device, CellPos { x, y }, 0);
    }
}

fn admit_build(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    device: crate::devices::DeviceId,
    anchor: CellPos,
    rotation: u8,
) -> bool {
    if mission.admit(CommandKind::QueueDevice(device)) != Admission::Accepted {
        return false;
    }
    let mut devices = std::mem::take(&mut world.devices);
    let mut fabrication = std::mem::take(&mut world.fabrication);
    let queued = devices
        .queue(world, device, anchor, rotation, &mut fabrication)
        .is_ok();
    let committed = if queued && mission.admit(CommandKind::CommitPlan) == Admission::Accepted {
        devices.commit_plan(world, &mut fabrication).is_ok()
    } else {
        false
    };
    world.devices = devices;
    world.fabrication = fabrication;
    committed
}

fn tick_scenario(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
    if id == MissionId::L03Firebreak
        && matches!(kind, ScenarioKind::Reference | ScenarioKind::Alternate)
        && world.devices.devices.iter().any(|device| {
            device.device == crate::devices::DeviceId::FlowTurbine && device.power_generated >= 2
        })
    {
        if let Some(pump) = world
            .devices
            .devices
            .iter()
            .find(|device| device.device == crate::devices::DeviceId::Pump)
            .map(|device| device.entity_id)
        {
            // Once the turbine has made the minimum useful delayed supply,
            // shed the bootstrap pump and give the relay a complete demand
            // allocation while the reaction stream settles.
            world.devices.set_enabled(pump, false);
        }
    }
    if id == MissionId::L02HoldingLine
        && matches!(kind, ScenarioKind::Reference | ScenarioKind::Alternate)
    {
        if world.devices.devices.iter().any(|device| {
            device.device == crate::devices::DeviceId::FlowTurbine && device.cumulative_power > 0
        }) {
            if let Some(reservoir) = world
                .devices
                .devices
                .iter()
                .find(|device| device.device == crate::devices::DeviceId::Reservoir)
                .map(|device| device.entity_id)
            {
                world.devices.set_enabled(reservoir, true);
            }
        }
        let trench_water = (31..=36)
            .flat_map(|x| (7..=9).map(move |y| CellPos { x, y }))
            .filter_map(|position| world.index(position))
            .flat_map(|index| world.cells[index].surface.iter())
            .filter(|entry| entry.fluid == FluidId::Water)
            .map(|entry| entry.volume_vu)
            .sum::<u32>();
        if let Some(reservoir) = world
            .devices
            .devices
            .iter_mut()
            .find(|device| device.device == crate::devices::DeviceId::Reservoir)
        {
            reservoir.setting_bp = if trench_water >= 6_500 || reservoir.stored_vu < 2_500 {
                10_000
            } else {
                5_000
            };
        }
    }
    if kind == ScenarioKind::Failure {
        match id {
            MissionId::L01FirstFlow => {
                world.inject(CellPos { x: 24, y: 8 }, FluidId::Water, 2_000);
            }
            MissionId::L02HoldingLine => {
                world.inject(CellPos { x: 26, y: 16 }, FluidId::Water, 2_000);
            }
            MissionId::L03Firebreak => {
                world.inject(CellPos { x: 37, y: 15 }, FluidId::Lava, 2_000);
            }
        }
    }
    apply_scheduled_events(world, id);
    world.tick();
    mission.on_tick(world);
}

fn objective_target(id: MissionId) -> u32 {
    crate::content::ContentRegistry::load()
        .ok()
        .and_then(|content| {
            content
                .mission(id.content_id())
                .map(|mission| mission.objective_min_vu)
        })
        .unwrap_or(0)
}

fn hash(world: &SimulationWorld, mission: &MissionState) -> u64 {
    let mut hash = 1469598103934665603u64;
    let Ok(bytes) = serde_json::to_vec(&(world, mission)) else {
        return 0;
    };
    for byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub fn run_all_scenarios() -> String {
    let mut passed = 0;
    let mut total = 0;
    for id in MissionId::ALL {
        for kind in [
            ScenarioKind::Reference,
            ScenarioKind::Alternate,
            ScenarioKind::Failure,
        ] {
            total += 1;
            let report = run_scenario(id, kind);
            let expected = match kind {
                ScenarioKind::Failure => report.terminal_failure,
                ScenarioKind::Reference | ScenarioKind::Alternate => report.terminal_success,
                ScenarioKind::InsufficientWaterRecovery => false,
            };
            if expected && report.continuation_matches && report.mass_balance_error == 0 {
                passed += 1;
            }
        }
    }
    let recovery = run_scenario(
        MissionId::L03Firebreak,
        ScenarioKind::InsufficientWaterRecovery,
    );
    total += 1;
    if recovery.objective_incomplete
        && !recovery.terminal_failure
        && recovery.continuation_matches
        && recovery.mass_balance_error == 0
    {
        passed += 1;
    }
    format!("{passed}/{total} deterministic campaign scenarios PASS")
}
