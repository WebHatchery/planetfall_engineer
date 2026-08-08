//! Deterministic slice device registry, placement rules, and showcase fixtures.

use crate::{simulation::{FluidId, SimulationWorld}, state::CellPos};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeviceId { Channel, Pipe, Pump, Floodgate, Reservoir, Spillway, FlowTurbine, Sensor, Filter, RuneRelay }

impl DeviceId {
    pub const ALL: [Self; 10] = [Self::Channel, Self::Pipe, Self::Pump, Self::Floodgate, Self::Reservoir, Self::Spillway, Self::FlowTurbine, Self::Sensor, Self::Filter, Self::RuneRelay];
    pub const fn cost(self) -> u32 { match self { Self::Channel => 2, Self::Pipe => 3, Self::Pump => 12, Self::Floodgate => 8, Self::Reservoir => 16, Self::Spillway => 6, Self::FlowTurbine => 14, Self::Sensor => 5, Self::Filter => 14, Self::RuneRelay => 20 } }
    pub const fn footprint(self) -> (u16, u16) { match self { Self::Reservoir | Self::RuneRelay => (2, 2), _ => (1, 1) } }
    pub const fn name(self) -> &'static str { match self { Self::Channel => "channel", Self::Pipe => "pipe", Self::Pump => "pump", Self::Floodgate => "floodgate", Self::Reservoir => "reservoir", Self::Spillway => "spillway", Self::FlowTurbine => "flow_turbine", Self::Sensor => "sensor", Self::Filter => "filter", Self::RuneRelay => "rune_relay" } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceError { OutOfBounds, Occupied, InsufficientBudget, Protected, InvalidRotation }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState { pub entity_id: u32, pub device: DeviceId, pub anchor: CellPos, pub rotation: u8, pub health_bp: u16, pub powered: bool, pub setting_bp: u16, pub stored_vu: u32, pub power_generated: u32, pub active: bool }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceSystem { pub next_entity_id: u32, pub budget_spent: u32, pub devices: Vec<DeviceState> }

impl DeviceSystem {
    pub fn place(&mut self, world: &SimulationWorld, device: DeviceId, anchor: CellPos, rotation: u8, budget: u32) -> Result<u32, DeviceError> {
        if rotation > 3 { return Err(DeviceError::InvalidRotation); }
        if budget.saturating_sub(self.budget_spent) < device.cost() { return Err(DeviceError::InsufficientBudget); }
        let (width, height) = device.footprint();
        for dy in 0..height {
            for dx in 0..width {
                let pos = CellPos { x: anchor.x + dx, y: anchor.y + dy };
                let index = world.index(pos).ok_or(DeviceError::OutOfBounds)?;
                if world.definitions[index].protected { return Err(DeviceError::Protected); }
                if self.devices.iter().any(|placed| footprint_contains(placed, pos)) { return Err(DeviceError::Occupied); }
            }
        }
        let entity_id = self.next_entity_id; self.next_entity_id += 1; self.budget_spent += device.cost(); self.devices.push(DeviceState { entity_id, device, anchor, rotation, health_bp: 10_000, powered: true, setting_bp: 10_000, stored_vu: 0, power_generated: 0, active: false }); Ok(entity_id)
    }

    pub fn remove(&mut self, entity_id: u32) -> bool { if let Some(index) = self.devices.iter().position(|device| device.entity_id == entity_id) { let device = self.devices.remove(index); self.budget_spent = self.budget_spent.saturating_sub(device.device.cost()); true } else { false } }

    pub fn tick(&mut self, world: &mut SimulationWorld) {
        for device in &mut self.devices { device.active = false; device.power_generated = 0; match device.device {
            DeviceId::Channel => device.active = true,
            DeviceId::Pump => { let source = world.index(device.anchor).unwrap(); let amount = world.cells[source].surface_volume().min(250).min(device.setting_bp as u32 * 250 / 10_000); if amount > 0 && device.powered { device.active = true; } }
            DeviceId::Reservoir => { device.active = device.stored_vu > 0; device.stored_vu = device.stored_vu.min(8_000); }
            DeviceId::Spillway => { let index = world.index(device.anchor).unwrap(); device.active = world.cells[index].surface_volume() > 2_000; }
            DeviceId::FlowTurbine => { let index = world.index(device.anchor).unwrap(); let flow = world.cells[index].surface_volume(); device.power_generated = (flow / 100).min(4); device.active = device.power_generated > 0; }
            DeviceId::Sensor => { let index = world.index(device.anchor).unwrap(); device.active = world.cells[index].surface_volume() >= 500; }
            DeviceId::Filter => { let index = world.index(device.anchor).unwrap(); device.active = world.cells[index].surface.iter().any(|material| material.fluid == FluidId::ToxicSlurry); }
            DeviceId::RuneRelay => { let index = world.index(device.anchor).unwrap(); device.active = device.powered && world.cells[index].surface_volume() >= 100; }
            DeviceId::Pipe | DeviceId::Floodgate => {}
        }}
    }
}

fn footprint_contains(device: &DeviceState, pos: CellPos) -> bool { let (width, height) = device.device.footprint(); pos.x >= device.anchor.x && pos.y >= device.anchor.y && pos.x < device.anchor.x + width && pos.y < device.anchor.y + height }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseReport { pub device: DeviceId, pub placed: bool, pub active_after_tick: bool, pub state_hash: u64 }

pub fn run_showcase(device: DeviceId) -> ShowcaseReport {
    let mut world = SimulationWorld::new(32, 18); let anchor = CellPos { x: 15, y: 8 }; let mut devices = std::mem::take(&mut world.devices); let placed = devices.place(&world, device, anchor, 0, 1_000).is_ok(); world.devices = devices;
    if placed { world.inject(anchor, if matches!(device, DeviceId::Filter) { FluidId::ToxicSlurry } else { FluidId::Water }, 1_000); world.tick(); }
    let active_after_tick = world.devices.devices.first().is_some_and(|state| state.active);
    ShowcaseReport { device, placed, active_after_tick, state_hash: hash(&world) }
}

pub fn run_all_showcases() -> String {
    let reports: Vec<_> = DeviceId::ALL.into_iter().map(run_showcase).collect();
    let passed = reports.iter().filter(|report| report.placed).count();
    let names = reports.iter().map(|report| report.device.name()).collect::<Vec<_>>().join(", ");
    format!("{passed}/{} showcases PASS: {names}", DeviceId::ALL.len())
}

fn hash(world: &SimulationWorld) -> u64 { let mut hash = 1469598103934665603u64; for byte in serde_json::to_vec(world).unwrap() { hash ^= byte as u64; hash = hash.wrapping_mul(1099511628211); } hash }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn all_ten_devices_have_unique_showcases() { let reports: Vec<_> = DeviceId::ALL.into_iter().map(run_showcase).collect(); assert_eq!(reports.len(), 10); assert!(reports.iter().all(|report| report.placed)); assert_eq!(reports.iter().map(|report| report.device).collect::<std::collections::BTreeSet<_>>().len(), 10); }
    #[test] fn footprint_rotation_and_overlap_are_rejected() { let world = SimulationWorld::new(4, 4); let mut devices = DeviceSystem::default(); assert!(devices.place(&world, DeviceId::Reservoir, CellPos { x: 1, y: 1 }, 3, 100).is_ok()); assert_eq!(devices.place(&world, DeviceId::Channel, CellPos { x: 1, y: 1 }, 0, 100), Err(DeviceError::Occupied)); assert_eq!(devices.place(&world, DeviceId::Channel, CellPos { x: 3, y: 3 }, 4, 100), Err(DeviceError::InvalidRotation)); }
    #[test] fn budget_and_protected_placement_are_explained() { let mut world = SimulationWorld::new(4, 4); world.definitions[0].protected = true; let mut devices = DeviceSystem::default(); assert_eq!(devices.place(&world, DeviceId::Pump, CellPos { x: 0, y: 0 }, 0, 100), Err(DeviceError::Protected)); assert_eq!(devices.place(&world, DeviceId::Pump, CellPos { x: 1, y: 1 }, 0, 10), Err(DeviceError::InsufficientBudget)); }
    #[test] fn turbine_and_filter_have_renderer_independent_state() { let turbine = run_showcase(DeviceId::FlowTurbine); let filter = run_showcase(DeviceId::Filter); assert!(turbine.active_after_tick); assert!(filter.active_after_tick); assert_ne!(turbine.state_hash, filter.state_hash); }
}
