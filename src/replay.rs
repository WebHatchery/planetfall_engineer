//! Deterministic campaign and failure-path replay harness.

use crate::{
    campaign::{apply_scheduled_events, load_campaign, seed_reference_materials},
    mission::{MissionId, MissionPhase, MissionState},
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
    pub mass_balance_error: i64,
    pub injected_vu: u64,
    pub drained_vu: u64,
    pub reacted_vu: u64,
    pub products_vu: u64,
}

pub fn run_scenario(id: MissionId, kind: ScenarioKind) -> ScenarioReport {
    let mut map = load_campaign(id);
    seed_reference_materials(&mut map);
    let mut mission = MissionState::new(id);
    mission.start();
    let initial_hash = hash(&map.world, &mission);
    seed_scenario(&mut map.world, id, kind);
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
        mass_balance_error,
        injected_vu: map.world.ledger.injected,
        drained_vu: map.world.ledger.drained,
        reacted_vu: map.world.ledger.reacted,
        products_vu: map.world.ledger.products,
    }
}

fn seed_scenario(world: &mut SimulationWorld, id: MissionId, kind: ScenarioKind) {
    match kind {
        ScenarioKind::Reference => seed_objective_water(world, id),
        ScenarioKind::Alternate => {
            let pos = match id {
                MissionId::L01FirstFlow => CellPos { x: 24, y: 9 },
                MissionId::L02HoldingLine => CellPos { x: 34, y: 8 },
                MissionId::L03Firebreak => CellPos { x: 23, y: 14 },
            };
            world.inject(pos, FluidId::Water, 6_000);
            if id == MissionId::L03Firebreak {
                world.inject(pos, FluidId::Lava, 6_000);
                let second = CellPos {
                    x: pos.x + 1,
                    y: pos.y,
                };
                world.inject(second, FluidId::Water, 6_000);
                world.inject(second, FluidId::Lava, 6_000);
            }
        }
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
                world.ledger.injected = world.ledger.injected.saturating_sub(6_000);
                world.inject(CellPos { x: 22, y: 14 }, FluidId::Water, 500);
                world.inject(CellPos { x: 22, y: 14 }, FluidId::Lava, 500);
            } else {
                seed_objective_water(world, id);
            }
        }
    }
}

fn tick_scenario(
    world: &mut SimulationWorld,
    mission: &mut MissionState,
    id: MissionId,
    kind: ScenarioKind,
) {
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

fn seed_objective_water(world: &mut SimulationWorld, id: MissionId) {
    let pos = match id {
        MissionId::L01FirstFlow => CellPos { x: 23, y: 8 },
        MissionId::L02HoldingLine => CellPos { x: 33, y: 8 },
        MissionId::L03Firebreak => CellPos { x: 22, y: 14 },
    };
    world.inject(pos, FluidId::Water, 6_000);
    if id == MissionId::L03Firebreak {
        world.inject(pos, FluidId::Lava, 6_000);
        let second = CellPos {
            x: pos.x + 1,
            y: pos.y,
        };
        world.inject(second, FluidId::Water, 6_000);
        world.inject(second, FluidId::Lava, 6_000);
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
