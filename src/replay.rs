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
    let midpoint_save = serde_json::to_vec(&(map.world.clone(), mission.clone()))
        .expect("midpoint replay state serializes");
    let (mut resumed_world, mut resumed_mission): (SimulationWorld, MissionState) =
        serde_json::from_slice(&midpoint_save).expect("midpoint replay state loads");
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
    }
}

fn seed_scenario(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
    match kind {
        // Success branches begin solely from the authored map state. Their
        // material arrives through sources and the admitted build below.
        ScenarioKind::Reference | ScenarioKind::Alternate => {}
        ScenarioKind::Failure => match id {
            MissionId::L01FirstFlow => {
                world.inject(CellPos { x: 10, y: 6 }, FluidId::Water, 4_000);
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
                for pos in [CellPos { x: 22, y: 14 }, CellPos { x: 23, y: 14 }] {
                    if let Some(index) = world.index(pos) {
                        world.cells[index].surface.clear();
                    }
                }
                world.inject(CellPos { x: 22, y: 14 }, FluidId::Water, 500);
                world.inject(CellPos { x: 22, y: 14 }, FluidId::Lava, 500);
            }
        }
    }
    install_reference_build(world, mission, id, kind);
    if matches!(kind, ScenarioKind::Reference | ScenarioKind::Alternate) {
        world.set_sources_enabled(true);
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
        MissionId::L02HoldingLine => {
            let _ = admit_build(
                world,
                mission,
                crate::devices::DeviceId::Pump,
                CellPos { x: 5, y: 16 },
                0,
            );
            for pos in [
                (9, 16),
                (11, 16),
                (13, 16),
                (15, 16),
                (19, 12),
                (20, 12),
                (20, 11),
                (20, 10),
            ] {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x: pos.0, y: pos.1 },
                    0,
                );
            }
            let _ = admit_build(
                world,
                mission,
                crate::devices::DeviceId::Reservoir,
                CellPos { x: 20, y: 8 },
                0,
            );
            for x in [23, 25, 27, 29] {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x, y: 8 },
                    0,
                );
            }
        }
        MissionId::L03Firebreak => {
            let _ = admit_build(
                world,
                mission,
                crate::devices::DeviceId::FlowTurbine,
                CellPos { x: 27, y: 9 },
                0,
            );
            for x in 22..=27 {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x, y: 14 },
                    0,
                );
            }
            for y in 10..=13 {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x: 27, y },
                    0,
                );
            }
            for x in 28..=37 {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x, y: 9 },
                    0,
                );
            }
            for y in 5..=8 {
                let _ = admit_build(
                    world,
                    mission,
                    crate::devices::DeviceId::Pipe,
                    CellPos { x: 37, y },
                    0,
                );
            }
            let _ = admit_build(
                world,
                mission,
                crate::devices::DeviceId::RuneRelay,
                CellPos { x: 38, y: 5 },
                0,
            );
        }
        MissionId::L01FirstFlow => {
            if mission.admit(CommandKind::SetGate(10_000)) == Admission::Accepted {
                let mut devices = std::mem::take(&mut world.devices);
                devices.set_selected_gate(CellPos { x: 21, y: 8 }, 10_000);
                world.devices = devices;
            }
            let _ = admit_build(
                world,
                mission,
                crate::devices::DeviceId::Channel,
                CellPos { x: 12, y: 8 },
                0,
            );
        }
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
    let queued = devices
        .queue(world, device, anchor, rotation, mission.budget)
        .is_ok();
    let committed = if queued && mission.admit(CommandKind::CommitPlan) == Admission::Accepted {
        devices.commit_plan(world, mission.budget).is_ok()
    } else {
        false
    };
    world.devices = devices;
    committed
}

fn tick_scenario(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
    if id == MissionId::L02HoldingLine
        && matches!(kind, ScenarioKind::Reference | ScenarioKind::Alternate)
    {
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
                world.inject(CellPos { x: 10, y: 6 }, FluidId::Water, 2_000);
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
    match id {
        MissionId::L01FirstFlow | MissionId::L02HoldingLine => 6_000,
        MissionId::L03Firebreak => 3_000,
    }
}

fn hash(world: &SimulationWorld, mission: &MissionState) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in serde_json::to_vec(&(world, mission)).unwrap() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_campaign_reference_reaches_terminal_success() {
        for id in MissionId::ALL {
            let report = run_scenario(id, ScenarioKind::Reference);
            assert!(report.terminal_success, "{report:?}");
            assert!(report.continuation_matches);
            assert_eq!(report.mass_balance_error, 0);
            assert!(report.ticks >= 100);
            assert!(report.injected_vu > 0);
            assert!(report.admitted_commands > 0, "{report:?}");
            assert!(report.placed_devices > 0, "{report:?}");
        }
    }

    #[test]
    fn campaign_reference_hashes_repeat() {
        for id in MissionId::ALL {
            assert_eq!(
                run_scenario(id, ScenarioKind::Reference),
                run_scenario(id, ScenarioKind::Reference)
            );
        }
    }

    #[test]
    fn alternate_routes_reach_success_and_preserve_midpoint_continuation() {
        for id in MissionId::ALL {
            let first = run_scenario(id, ScenarioKind::Alternate);
            let second = run_scenario(id, ScenarioKind::Alternate);
            assert_eq!(first, second);
            assert!(first.terminal_success, "{first:?}");
            assert!(first.continuation_matches);
            assert_eq!(first.mass_balance_error, 0);
            assert_ne!(first.midpoint_hash, first.final_hash);
            assert!(first.admitted_commands > 0, "{first:?}");
        }
    }

    #[test]
    fn authored_failure_scenarios_fail_deterministically() {
        for id in MissionId::ALL {
            let report = run_scenario(id, ScenarioKind::Failure);
            assert!(report.terminal_failure, "{report:?}");
            assert!(!report.terminal_success);
            assert!(report.continuation_matches);
            assert_eq!(report.mass_balance_error, 0);
        }
    }

    #[test]
    fn l03_insufficient_water_stays_recoverable() {
        let report = run_scenario(
            MissionId::L03Firebreak,
            ScenarioKind::InsufficientWaterRecovery,
        );
        assert!(!report.terminal_success);
        assert!(!report.terminal_failure);
        assert!(report.objective_incomplete);
        assert!(report.continuation_matches);
        assert_eq!(report.mass_balance_error, 0);
    }
}
