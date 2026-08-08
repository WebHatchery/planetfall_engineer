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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowcaseMap { pub map_id: &'static str, pub device: DeviceId }

pub const SHOWCASE_MAPS: [ShowcaseMap; 10] = [
    ShowcaseMap { map_id: "device_channel", device: DeviceId::Channel },
    ShowcaseMap { map_id: "device_pipe", device: DeviceId::Pipe },
    ShowcaseMap { map_id: "device_pump", device: DeviceId::Pump },
    ShowcaseMap { map_id: "device_floodgate", device: DeviceId::Floodgate },
    ShowcaseMap { map_id: "device_reservoir", device: DeviceId::Reservoir },
    ShowcaseMap { map_id: "device_spillway", device: DeviceId::Spillway },
    ShowcaseMap { map_id: "device_flow_turbine", device: DeviceId::FlowTurbine },
    ShowcaseMap { map_id: "device_sensor", device: DeviceId::Sensor },
    ShowcaseMap { map_id: "device_filter", device: DeviceId::Filter },
    ShowcaseMap { map_id: "device_rune_relay", device: DeviceId::RuneRelay },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceError { OutOfBounds, Occupied, InsufficientBudget, Protected, InvalidRotation }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState { pub entity_id: u32, pub device: DeviceId, pub anchor: CellPos, pub rotation: u8, pub health_bp: u16, pub powered: bool, pub setting_bp: u16, pub stored_vu: u32, pub power_generated: u32, pub active: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedPlan { pub plan_id: u32, pub device: DeviceId, pub anchor: CellPos, pub rotation: u8, pub cost: u32 }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceSystem { pub next_entity_id: u32, pub budget_spent: u32, pub reserved_budget: u32, pub next_plan_id: u32, pub queued: Vec<QueuedPlan>, pub devices: Vec<DeviceState> }

impl DeviceSystem {
    pub fn place(&mut self, world: &SimulationWorld, device: DeviceId, anchor: CellPos, rotation: u8, budget: u32) -> Result<u32, DeviceError> {
        if rotation > 3 { return Err(DeviceError::InvalidRotation); }
        if budget.saturating_sub(self.budget_spent) < device.cost() { return Err(DeviceError::InsufficientBudget); }
        validate_placement(world, &self.devices, &[], device, anchor, rotation)?;
        let entity_id = self.next_entity_id; self.next_entity_id += 1; self.budget_spent += device.cost(); self.devices.push(DeviceState { entity_id, device, anchor, rotation, health_bp: 10_000, powered: true, setting_bp: 10_000, stored_vu: 0, power_generated: 0, active: false }); Ok(entity_id)
    }

    pub fn queue(&mut self, world: &SimulationWorld, device: DeviceId, anchor: CellPos, rotation: u8, budget: u32) -> Result<u32, DeviceError> {
        if self.reserved_budget + device.cost() > budget.saturating_sub(self.budget_spent) { return Err(DeviceError::InsufficientBudget); }
        validate_placement(world, &self.devices, &self.queued, device, anchor, rotation)?;
        let plan_id = self.next_plan_id; self.next_plan_id += 1; self.reserved_budget += device.cost(); self.queued.push(QueuedPlan { plan_id, device, anchor, rotation, cost: device.cost() }); Ok(plan_id)
    }

    pub fn cancel_last_plan(&mut self) -> bool { if let Some(plan) = self.queued.pop() { self.reserved_budget = self.reserved_budget.saturating_sub(plan.cost); true } else { false } }

    pub fn commit_plan(&mut self, world: &SimulationWorld, budget: u32) -> Result<Vec<u32>, DeviceError> {
        let plans = self.queued.clone();
        for plan in &plans { if self.devices.iter().any(|placed| footprints_overlap(placed.device, placed.anchor, plan.device, plan.anchor)) { return Err(DeviceError::Occupied); } }
        let mut committed = Vec::with_capacity(plans.len());
        for plan in plans { committed.push(self.place(world, plan.device, plan.anchor, plan.rotation, budget)?); }
        self.queued.clear(); self.reserved_budget = 0; Ok(committed)
    }

    pub fn remove(&mut self, entity_id: u32) -> bool { if let Some(index) = self.devices.iter().position(|device| device.entity_id == entity_id) { let device = self.devices.remove(index); self.budget_spent = self.budget_spent.saturating_sub(device.device.cost()); true } else { false } }

    pub fn set_selected_gate(&mut self, anchor: CellPos, setting_bp: u16) -> bool { if let Some(gate) = self.devices.iter_mut().find(|device| device.device == DeviceId::Floodgate && device.anchor == anchor) { gate.setting_bp = setting_bp.min(10_000); true } else { false } }

    pub fn surface_flow_factor(&self, source: CellPos, destination: CellPos) -> u32 { self.devices.iter().find_map(|device| { if device.device != DeviceId::Floodgate { return None; } let delta = (destination.x as i16 - source.x as i16, destination.y as i16 - source.y as i16); let direction = direction(device.rotation); let source_side = source == device.anchor || destination == device.anchor; let matches_edge = delta == direction || delta == (-direction.0, -direction.1); (source_side && matches_edge).then_some(device.setting_bp as u32) }).unwrap_or(10_000) }

    pub fn tick(&mut self, world: &mut SimulationWorld) {
        for device in &mut self.devices { device.active = false; device.power_generated = 0; match device.device {
            DeviceId::Channel => device.active = true,
            DeviceId::Pump => { let source = world.index(device.anchor).unwrap(); let amount = world.cells[source].surface_volume().min(250).min(device.setting_bp as u32 * 250 / 10_000); if amount > 0 && device.powered && transfer_surface(world, device.anchor, direction(device.rotation), amount) > 0 { device.active = true; } }
            DeviceId::Reservoir => { let index = world.index(device.anchor).unwrap(); let accepted = world.cells[index].surface_volume().min(8_000u32.saturating_sub(device.stored_vu)); if accepted > 0 { let fluid = world.cells[index].surface.first().map(|entry| entry.fluid); if let Some(fluid) = fluid { remove_fluid(&mut world.cells[index].surface, fluid, accepted); device.stored_vu += accepted; device.active = true; } } }
            DeviceId::Spillway => { let index = world.index(device.anchor).unwrap(); let depth = world.cells[index].surface_volume(); if depth > 2_000 { device.active = transfer_surface(world, device.anchor, direction(device.rotation), (depth - 2_000).min(500)) > 0; } }
            DeviceId::FlowTurbine => { let index = world.index(device.anchor).unwrap(); let flow = world.cells[index].surface_volume(); device.power_generated = (flow / 100).min(4); device.active = device.power_generated > 0; }
            DeviceId::Sensor => { let index = world.index(device.anchor).unwrap(); device.active = world.cells[index].surface_volume() >= 500; }
            DeviceId::Filter => { let index = world.index(device.anchor).unwrap(); if let Some(material) = world.cells[index].surface.iter_mut().find(|material| material.fluid == FluidId::ToxicSlurry) { material.contamination_bp = material.contamination_bp.saturating_sub(2_500); device.active = true; } }
            DeviceId::RuneRelay => { let index = world.index(device.anchor).unwrap(); device.active = device.powered && world.cells[index].surface_volume() >= 100; }
            DeviceId::Pipe | DeviceId::Floodgate => {}
        }}
    }
}

fn direction(rotation: u8) -> (i16, i16) { match rotation % 4 { 0 => (1, 0), 1 => (0, 1), 2 => (-1, 0), _ => (0, -1) } }
fn transfer_surface(world: &mut SimulationWorld, source: CellPos, (dx, dy): (i16, i16), amount: u32) -> u32 { let x = source.x as i16 + dx; let y = source.y as i16 + dy; if x < 0 || y < 0 { return 0; } let destination = CellPos { x: x as u16, y: y as u16 }; let Some(source_index) = world.index(source) else { return 0; }; let Some(destination_index) = world.index(destination) else { return 0; }; let Some(entry) = world.cells[source_index].surface.first().cloned() else { return 0; }; let moved = amount.min(entry.volume_vu); remove_fluid(&mut world.cells[source_index].surface, entry.fluid, moved); world.cells[destination_index].add_surface(crate::simulation::FluidEntry { fluid: entry.fluid, volume_vu: moved, temperature_dk: entry.temperature_dk, contamination_bp: entry.contamination_bp }); moved }
fn remove_fluid(entries: &mut Vec<crate::simulation::FluidEntry>, fluid: FluidId, amount: u32) { if let Some(entry) = entries.iter_mut().find(|entry| entry.fluid == fluid) { entry.volume_vu -= amount.min(entry.volume_vu); } entries.retain(|entry| entry.volume_vu > 0); }

fn validate_placement(world: &SimulationWorld, devices: &[DeviceState], queued: &[QueuedPlan], device: DeviceId, anchor: CellPos, rotation: u8) -> Result<(), DeviceError> {
    if rotation > 3 { return Err(DeviceError::InvalidRotation); }
    let (width, height) = device.footprint();
    for dy in 0..height {
        for dx in 0..width {
            let pos = CellPos { x: anchor.x + dx, y: anchor.y + dy };
            let index = world.index(pos).ok_or(DeviceError::OutOfBounds)?;
            if world.definitions[index].protected { return Err(DeviceError::Protected); }
            if devices.iter().any(|placed| footprint_contains(placed.device, placed.anchor, pos)) || queued.iter().any(|plan| footprint_contains(plan.device, plan.anchor, pos)) { return Err(DeviceError::Occupied); }
        }
    }
    Ok(())
}
fn footprint_contains(device: DeviceId, anchor: CellPos, pos: CellPos) -> bool { let (width, height) = device.footprint(); pos.x >= anchor.x && pos.y >= anchor.y && pos.x < anchor.x + width && pos.y < anchor.y + height }
fn footprints_overlap(first: DeviceId, first_anchor: CellPos, second: DeviceId, second_anchor: CellPos) -> bool { let (width, height) = first.footprint(); (0..height).any(|dy| (0..width).any(|dx| footprint_contains(second, second_anchor, CellPos { x: first_anchor.x + dx, y: first_anchor.y + dy }))) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseReport { pub device: DeviceId, pub placed: bool, pub active_after_tick: bool, pub state_hash: u64 }

pub fn run_showcase(device: DeviceId) -> ShowcaseReport {
    let mut world = SimulationWorld::new(32, 18); let anchor = CellPos { x: 15, y: 8 }; let mut devices = std::mem::take(&mut world.devices); let placed = devices.place(&world, device, anchor, 0, 1_000).is_ok(); world.devices = devices;
    if placed { world.inject(anchor, if matches!(device, DeviceId::Filter) { FluidId::ToxicSlurry } else { FluidId::Water }, 1_000); world.tick(); }
    let active_after_tick = world.devices.devices.first().is_some_and(|state| state.active);
    ShowcaseReport { device, placed, active_after_tick, state_hash: hash(&world) }
}

pub fn run_all_showcases() -> String {
    let reports: Vec<_> = SHOWCASE_MAPS.into_iter().map(|showcase| run_showcase(showcase.device)).collect();
    let passed = reports.iter().filter(|report| report.placed).count();
    let names = reports.iter().map(|report| report.device.name()).collect::<Vec<_>>().join(", ");
    format!("{passed}/{} showcases PASS: {names}", DeviceId::ALL.len())
}

fn hash(world: &SimulationWorld) -> u64 { let mut hash = 1469598103934665603u64; for byte in serde_json::to_vec(world).unwrap() { hash ^= byte as u64; hash = hash.wrapping_mul(1099511628211); } hash }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn all_ten_devices_have_unique_showcases() { let reports: Vec<_> = DeviceId::ALL.into_iter().map(run_showcase).collect(); assert_eq!(reports.len(), 10); assert!(reports.iter().all(|report| report.placed)); assert_eq!(reports.iter().map(|report| report.device).collect::<std::collections::BTreeSet<_>>().len(), 10); }
    #[test] fn showcase_index_has_exactly_one_named_map_per_device() { assert_eq!(SHOWCASE_MAPS.len(), DeviceId::ALL.len()); assert_eq!(SHOWCASE_MAPS.iter().map(|showcase| showcase.device).collect::<std::collections::BTreeSet<_>>().len(), 10); assert!(SHOWCASE_MAPS.iter().all(|showcase| showcase.map_id == format!("device_{}", showcase.device.name()))); }
    #[test] fn footprint_rotation_and_overlap_are_rejected() { let world = SimulationWorld::new(4, 4); let mut devices = DeviceSystem::default(); assert!(devices.place(&world, DeviceId::Reservoir, CellPos { x: 1, y: 1 }, 3, 100).is_ok()); assert_eq!(devices.place(&world, DeviceId::Channel, CellPos { x: 1, y: 1 }, 0, 100), Err(DeviceError::Occupied)); assert_eq!(devices.place(&world, DeviceId::Channel, CellPos { x: 3, y: 3 }, 4, 100), Err(DeviceError::InvalidRotation)); }
    #[test] fn budget_and_protected_placement_are_explained() { let mut world = SimulationWorld::new(4, 4); world.definitions[0].protected = true; let mut devices = DeviceSystem::default(); assert_eq!(devices.place(&world, DeviceId::Pump, CellPos { x: 0, y: 0 }, 0, 100), Err(DeviceError::Protected)); assert_eq!(devices.place(&world, DeviceId::Pump, CellPos { x: 1, y: 1 }, 0, 10), Err(DeviceError::InsufficientBudget)); }
    #[test] fn turbine_and_filter_have_renderer_independent_state() { let turbine = run_showcase(DeviceId::FlowTurbine); let filter = run_showcase(DeviceId::Filter); assert!(turbine.active_after_tick); assert!(filter.active_after_tick); assert_ne!(turbine.state_hash, filter.state_hash); }
    #[test] fn pump_moves_water_in_rotated_direction() { let mut world = SimulationWorld::new(3, 1); let anchor = CellPos { x: 0, y: 0 }; let mut placed = std::mem::take(&mut world.devices); placed.place(&world, DeviceId::Pump, anchor, 0, 100).unwrap(); world.devices = placed; world.inject(anchor, FluidId::Water, 500); let mut devices = std::mem::take(&mut world.devices); devices.tick(&mut world); assert_eq!(world.cells[0].surface_volume(), 250); assert_eq!(world.cells[1].surface_volume(), 250); assert!(devices.devices[0].active); world.devices = devices; }
    #[test] fn filter_removes_contamination_without_losing_volume() { let mut world = SimulationWorld::new(2, 1); let mut devices = std::mem::take(&mut world.devices); devices.place(&world, DeviceId::Filter, CellPos { x: 0, y: 0 }, 0, 100).unwrap(); world.devices = devices; world.inject(CellPos { x: 0, y: 0 }, FluidId::ToxicSlurry, 500); let mut devices = std::mem::take(&mut world.devices); devices.tick(&mut world); let entry = &world.cells[0].surface[0]; assert_eq!(entry.volume_vu, 500); assert_eq!(entry.contamination_bp, 7_500); assert!(devices.devices[0].active); world.devices = devices; }
    #[test] fn queued_plans_reserve_budget_and_commit_atomically() { let mut world = SimulationWorld::new(4, 2); let mut devices = std::mem::take(&mut world.devices); assert_eq!(devices.queue(&world, DeviceId::Channel, CellPos { x: 0, y: 0 }, 0, 5), Ok(0)); assert_eq!(devices.queue(&world, DeviceId::Pipe, CellPos { x: 1, y: 0 }, 0, 5), Ok(1)); assert_eq!(devices.reserved_budget, 5); let committed = devices.commit_plan(&world, 5).unwrap(); assert_eq!(committed, vec![0, 1]); assert!(devices.queued.is_empty()); assert_eq!(devices.budget_spent, 5); world.devices = devices; }
    #[test] fn cancelling_a_plan_releases_reserved_budget() { let world = SimulationWorld::new(2, 1); let mut devices = DeviceSystem::default(); devices.queue(&world, DeviceId::Channel, CellPos { x: 0, y: 0 }, 0, 10).unwrap(); assert!(devices.cancel_last_plan()); assert_eq!(devices.reserved_budget, 0); assert!(!devices.cancel_last_plan()); }
    #[test] fn floodgate_setting_changes_edge_flow_factor() { let world = SimulationWorld::new(2, 1); let mut devices = DeviceSystem::default(); devices.place(&world, DeviceId::Floodgate, CellPos { x: 0, y: 0 }, 0, 100).unwrap(); assert_eq!(devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }), 10_000); devices.set_selected_gate(CellPos { x: 0, y: 0 }, 5_000); assert_eq!(devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }), 5_000); devices.set_selected_gate(CellPos { x: 0, y: 0 }, 0); assert_eq!(devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }), 0); }
    #[test] fn closed_gate_stops_surface_transfer_until_reopened() { let mut world = SimulationWorld::new(2, 1); world.cells[0].height_hu = 2_000; world.cells[1].height_hu = 0; let mut devices = std::mem::take(&mut world.devices); devices.place(&world, DeviceId::Floodgate, CellPos { x: 0, y: 0 }, 0, 100).unwrap(); devices.set_selected_gate(CellPos { x: 0, y: 0 }, 0); world.devices = devices; world.inject(CellPos { x: 0, y: 0 }, FluidId::Water, 1_000); world.tick(); assert_eq!(world.cells[1].surface_volume(), 0); let mut devices = std::mem::take(&mut world.devices); devices.set_selected_gate(CellPos { x: 0, y: 0 }, 10_000); world.devices = devices; world.inject(CellPos { x: 0, y: 0 }, FluidId::Water, 1_000); world.tick(); assert!(world.cells[1].surface_volume() > 0); }
}
