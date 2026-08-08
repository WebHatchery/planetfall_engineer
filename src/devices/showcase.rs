//! Deterministic device showcase fixtures kept separate from placement logic.

use super::{DeviceId, SHOWCASE_MAPS};
use crate::{
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseReport {
    pub device: DeviceId,
    pub placed: bool,
    pub active_after_tick: bool,
    pub mass_balance_ok: bool,
    pub state_hash: u64,
}

pub fn run_showcase(device: DeviceId) -> ShowcaseReport {
    let (world, placed) = build_showcase_world(device);
    let world = if placed {
        let mut world = world;
        world.tick();
        world
    } else {
        world
    };
    ShowcaseReport {
        device,
        placed,
        active_after_tick: world
            .devices
            .devices
            .first()
            .is_some_and(|state| state.active),
        mass_balance_ok: world.mass_balance_error() == 0,
        state_hash: hash(&world),
    }
}

pub fn showcase_world(device: DeviceId) -> SimulationWorld {
    build_showcase_world(device).0
}

fn build_showcase_world(device: DeviceId) -> (SimulationWorld, bool) {
    let mut world = SimulationWorld::new(32, 18);
    if device == DeviceId::FlowTurbine {
        for definition in &mut world.definitions {
            definition.ambient_temperature_dk = 4_730;
        }
    }
    let anchor = CellPos { x: 15, y: 8 };
    let mut devices = std::mem::take(&mut world.devices);
    let placed = devices.place(&world, device, anchor, 0, 1_000).is_ok();
    world.devices = devices;
    if placed {
        let fluid = if matches!(device, DeviceId::FlowTurbine) {
            FluidId::Steam
        } else if matches!(device, DeviceId::Filter) {
            FluidId::ToxicSlurry
        } else {
            FluidId::Water
        };
        world.inject(anchor, fluid, 1_000);
        world.tick();
    }
    (world, placed)
}

pub fn run_all_showcases() -> String {
    let reports: Vec<_> = SHOWCASE_MAPS
        .into_iter()
        .map(|showcase| run_showcase(showcase.device))
        .collect();
    let passed = reports
        .iter()
        .filter(|report| report.placed && report.mass_balance_ok)
        .count();
    let names = reports
        .iter()
        .map(|report| report.device.name())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{passed}/{} showcases PASS: {names}", DeviceId::ALL.len())
}

fn hash(world: &SimulationWorld) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in serde_json::to_vec(world).unwrap() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}
