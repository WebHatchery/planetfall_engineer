//! Executable verification fixtures for the slice's material laboratory.

use crate::{simulation::{FluidId, SimulationWorld}, state::CellPos};

pub const LAB_WIDTH: u16 = 64;
pub const LAB_HEIGHT: u16 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidBay {
    pub name: &'static str,
    pub fluid: Option<FluidId>,
    pub origin: CellPos,
}

pub const LAB_BAYS: [FluidBay; 9] = [
    FluidBay { name: "north_west_water", fluid: Some(FluidId::Water), origin: CellPos { x: 2, y: 2 } },
    FluidBay { name: "north_brine_reserved", fluid: None, origin: CellPos { x: 27, y: 2 } },
    FluidBay { name: "north_east_lava", fluid: Some(FluidId::Lava), origin: CellPos { x: 52, y: 2 } },
    FluidBay { name: "west_north_steam", fluid: Some(FluidId::Steam), origin: CellPos { x: 2, y: 20 } },
    FluidBay { name: "east_north_cryofluid_reserved", fluid: None, origin: CellPos { x: 52, y: 20 } },
    FluidBay { name: "west_south_acid_reserved", fluid: None, origin: CellPos { x: 2, y: 38 } },
    FluidBay { name: "east_south_slurry", fluid: Some(FluidId::ToxicSlurry), origin: CellPos { x: 52, y: 38 } },
    FluidBay { name: "south_west_nutrient_reserved", fluid: None, origin: CellPos { x: 20, y: 38 } },
    FluidBay { name: "south_east_aether_reserved", fluid: None, origin: CellPos { x: 38, y: 38 } },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabReport {
    pub tick: u64,
    pub state_hash: u64,
    pub active_bays: usize,
    pub reserved_bays_empty: bool,
    pub reaction_events: usize,
    pub mass_balance_ok: bool,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub struct FluidsLab {
    pub world: SimulationWorld,
}

impl FluidsLab {
    pub fn new() -> Self {
        let mut world = SimulationWorld::new(LAB_WIDTH, LAB_HEIGHT);
        for bay in LAB_BAYS {
            let origin = bay.origin;
            for dy in 0..4 { for dx in 0..4 { let pos = CellPos { x: origin.x + dx, y: origin.y + dy }; if let Some(index) = world.index(pos) { world.cells[index].sealed = true; } } }
            if bay.fluid.is_none() { continue; }
            let source = CellPos { x: origin.x + 1, y: origin.y + 1 };
            let source_index = world.index(source).unwrap();
            world.cells[source_index].height_hu = 2_000;
            let lower = CellPos { x: origin.x + 6, y: origin.y + 1 };
            let lower_index = world.index(lower).unwrap();
            world.cells[lower_index].height_hu = 0;
            world.inject(source, bay.fluid.unwrap(), 2_000);
        }
        Self::seed_interaction_lanes(&mut world);
        Self { world }
    }

    fn seed_interaction_lanes(world: &mut SimulationWorld) {
        let water_lava = CellPos { x: 29, y: 20 };
        let lava_slurry = CellPos { x: 34, y: 20 };
        let steam_condense = CellPos { x: 29, y: 26 };
        let non_reaction = CellPos { x: 34, y: 26 };
        world.inject(water_lava, FluidId::Water, 1_000);
        world.inject(water_lava, FluidId::Lava, 1_000);
        world.inject(lava_slurry, FluidId::Lava, 1_000);
        world.inject(lava_slurry, FluidId::ToxicSlurry, 1_000);
        world.inject(steam_condense, FluidId::Steam, 1_000);
        world.inject(non_reaction, FluidId::Water, 1_000);
        world.inject(non_reaction, FluidId::ToxicSlurry, 1_000);
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    pub fn automatic_scenario(&mut self) -> LabReport {
        self.reset();
        let mut reaction_events = 0;
        for _ in 0..300 {
            self.world.tick();
            reaction_events += self.world.events.iter().filter(|event| matches!(event, crate::simulation::SimEvent::MaterialReacted { .. })).count();
        }
        let reserved_bays_empty = LAB_BAYS.iter().filter(|bay| bay.fluid.is_none()).all(|bay| {
            let index = self.world.index(bay.origin).unwrap();
            self.world.cells[index].surface.is_empty() && self.world.cells[index].airborne.is_empty()
        });
        LabReport { tick: self.world.tick, state_hash: state_hash(&self.world), active_bays: LAB_BAYS.iter().filter(|bay| bay.fluid.is_some()).count(), reserved_bays_empty, reaction_events, mass_balance_ok: self.world.mass_balance_error() == 0, passed: reserved_bays_empty && self.world.tick == 300 && self.world.ledger.injected > 0 && self.world.mass_balance_error() == 0 }
    }
}

fn state_hash(world: &SimulationWorld) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in serde_json::to_vec(world).expect("verification world serializes") { hash ^= byte as u64; hash = hash.wrapping_mul(1099511628211); }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn lab_has_four_active_and_five_reserved_bays() { assert_eq!(LAB_BAYS.iter().filter(|bay| bay.fluid.is_some()).count(), 4); assert_eq!(LAB_BAYS.iter().filter(|bay| bay.fluid.is_none()).count(), 5); }
    #[test] fn lab_automatic_scenario_passes() { let report = FluidsLab::new().automatic_scenario(); assert!(report.passed, "{report:?}"); assert!(report.reaction_events > 0); }
    #[test] fn lab_replay_hash_is_stable() { let first = FluidsLab::new().automatic_scenario(); let second = FluidsLab::new().automatic_scenario(); assert_eq!(first, second); }
    #[test] fn reset_restores_initial_hash() { let mut lab = FluidsLab::new(); let initial = state_hash(&lab.world); lab.world.tick(); lab.reset(); assert_eq!(state_hash(&lab.world), initial); }
}
