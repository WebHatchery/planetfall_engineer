//! Deterministic campaign reference-replay harness.

use crate::{
    campaign::{load_campaign, seed_reference_materials},
    mission::{MissionId, MissionPhase, MissionState},
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayReport {
    pub id: MissionId,
    pub initial_hash: u64,
    pub final_hash: u64,
    pub ticks: u64,
    pub saved_continuation_matches: bool,
    pub terminal_success: bool,
    pub mass_balance_ok: bool,
    pub mass_balance_error: i64,
}

pub fn run_reference(id: MissionId) -> ReplayReport {
    let mut map = load_campaign(id);
    seed_reference_materials(&mut map);
    let mut mission = MissionState::new(id);
    mission.start();
    let initial_hash = hash(&map.world, &mission);
    seed_objective_water(&mut map.world, id);
    for _ in 0..50 {
        map.world.tick();
        mission.on_tick(&map.world);
    }
    let mut resumed_world = map.world.clone();
    let mut resumed_mission = mission.clone();
    while mission.phase == MissionPhase::Active && mission.tick < 300 {
        map.world.tick();
        mission.on_tick(&map.world);
    }
    let final_hash = hash(&map.world, &mission);
    while resumed_mission.phase == MissionPhase::Active && resumed_mission.tick < 300 {
        resumed_world.tick();
        resumed_mission.on_tick(&resumed_world);
    }
    let saved_continuation_matches = final_hash == hash(&resumed_world, &resumed_mission);
    let mass_balance_error = map.world.mass_balance_error();
    ReplayReport {
        id,
        initial_hash,
        final_hash,
        ticks: mission.tick,
        saved_continuation_matches,
        terminal_success: mission.phase == MissionPhase::Success,
        mass_balance_ok: mass_balance_error == 0,
        mass_balance_error,
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

pub fn run_all_references() -> String {
    let reports: Vec<_> = MissionId::ALL.into_iter().map(run_reference).collect();
    let passed = reports
        .iter()
        .filter(|report| {
            report.terminal_success && report.saved_continuation_matches && report.mass_balance_ok
        })
        .count();
    format!("{passed}/{} campaign references PASS", reports.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_campaign_reference_reaches_terminal_success() {
        for id in MissionId::ALL {
            let report = run_reference(id);
            assert!(report.terminal_success, "{report:?}");
            assert!(report.saved_continuation_matches);
            assert!(report.mass_balance_ok);
            assert!(report.ticks >= 100);
        }
    }
    #[test]
    fn campaign_reference_hashes_repeat() {
        for id in MissionId::ALL {
            assert_eq!(run_reference(id), run_reference(id));
        }
    }
}
