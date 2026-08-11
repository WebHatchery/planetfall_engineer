//! Deterministic slice device registry, placement rules, and showcase fixtures.

use crate::{
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};
use serde::{Deserialize, Serialize};

mod network;
mod showcase;
use network::{
    direction, footprint_outlet, pipe_connected, pipe_endpoint, step, transfer_surface,
    transfer_surface_to,
};
pub use showcase::{run_all_showcases, showcase_world};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeviceId {
    Channel,
    Pipe,
    Pump,
    Floodgate,
    Reservoir,
    Spillway,
    FlowTurbine,
    Sensor,
    Filter,
    RuneRelay,
}

impl DeviceId {
    pub const ALL: [Self; 10] = [
        Self::Channel,
        Self::Pipe,
        Self::Pump,
        Self::Floodgate,
        Self::Reservoir,
        Self::Spillway,
        Self::FlowTurbine,
        Self::Sensor,
        Self::Filter,
        Self::RuneRelay,
    ];
    pub const fn cost(self) -> u32 {
        match self {
            Self::Channel => 2,
            Self::Pipe => 3,
            Self::Pump => 12,
            Self::Floodgate => 8,
            Self::Reservoir => 16,
            Self::Spillway => 6,
            Self::FlowTurbine => 14,
            Self::Sensor => 5,
            Self::Filter => 14,
            Self::RuneRelay => 20,
        }
    }
    pub const fn footprint(self) -> (u16, u16) {
        match self {
            Self::Reservoir | Self::RuneRelay => (2, 2),
            _ => (1, 1),
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Pipe => "pipe",
            Self::Pump => "pump",
            Self::Floodgate => "floodgate",
            Self::Reservoir => "reservoir",
            Self::Spillway => "spillway",
            Self::FlowTurbine => "flow_turbine",
            Self::Sensor => "sensor",
            Self::Filter => "filter",
            Self::RuneRelay => "rune_relay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowcaseMap {
    pub map_id: &'static str,
    pub device: DeviceId,
}

pub const SHOWCASE_MAPS: [ShowcaseMap; 10] = [
    ShowcaseMap {
        map_id: "device_channel",
        device: DeviceId::Channel,
    },
    ShowcaseMap {
        map_id: "device_pipe",
        device: DeviceId::Pipe,
    },
    ShowcaseMap {
        map_id: "device_pump",
        device: DeviceId::Pump,
    },
    ShowcaseMap {
        map_id: "device_floodgate",
        device: DeviceId::Floodgate,
    },
    ShowcaseMap {
        map_id: "device_reservoir",
        device: DeviceId::Reservoir,
    },
    ShowcaseMap {
        map_id: "device_spillway",
        device: DeviceId::Spillway,
    },
    ShowcaseMap {
        map_id: "device_flow_turbine",
        device: DeviceId::FlowTurbine,
    },
    ShowcaseMap {
        map_id: "device_sensor",
        device: DeviceId::Sensor,
    },
    ShowcaseMap {
        map_id: "device_filter",
        device: DeviceId::Filter,
    },
    ShowcaseMap {
        map_id: "device_rune_relay",
        device: DeviceId::RuneRelay,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceError {
    OutOfBounds,
    Occupied,
    InsufficientBudget,
    Protected,
    InvalidRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub entity_id: u32,
    pub device: DeviceId,
    pub anchor: CellPos,
    pub rotation: u8,
    pub health_bp: u16,
    pub powered: bool,
    pub setting_bp: u16,
    pub stored_vu: u32,
    #[serde(default)]
    pub stored_fluid: Option<FluidId>,
    pub power_generated: u32,
    #[serde(default)]
    pub cumulative_power: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedPlan {
    pub plan_id: u32,
    pub device: DeviceId,
    pub anchor: CellPos,
    pub rotation: u8,
    pub cost: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceSystem {
    pub next_entity_id: u32,
    pub budget_spent: u32,
    pub reserved_budget: u32,
    pub next_plan_id: u32,
    pub queued: Vec<QueuedPlan>,
    pub devices: Vec<DeviceState>,
}

impl DeviceSystem {
    /// Install map-authored infrastructure without charging the player budget.
    /// Fixtures still occupy their footprint and run through the same device
    /// simulation as player-built equipment.
    pub fn install_fixture(
        &mut self,
        world: &SimulationWorld,
        device: DeviceId,
        anchor: CellPos,
        rotation: u8,
    ) -> Result<u32, DeviceError> {
        validate_placement(world, &self.devices, &[], device, anchor, rotation)?;
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        self.devices.push(DeviceState {
            entity_id,
            device,
            anchor,
            rotation,
            health_bp: 10_000,
            powered: true,
            setting_bp: 10_000,
            stored_vu: 0,
            stored_fluid: None,
            power_generated: 0,
            cumulative_power: 0,
            active: false,
        });
        Ok(entity_id)
    }

    pub fn place(
        &mut self,
        world: &SimulationWorld,
        device: DeviceId,
        anchor: CellPos,
        rotation: u8,
        budget: u32,
    ) -> Result<u32, DeviceError> {
        if rotation > 3 {
            return Err(DeviceError::InvalidRotation);
        }
        if budget.saturating_sub(self.budget_spent) < device.cost() {
            return Err(DeviceError::InsufficientBudget);
        }
        validate_placement(world, &self.devices, &[], device, anchor, rotation)?;
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        self.budget_spent += device.cost();
        self.devices.push(DeviceState {
            entity_id,
            device,
            anchor,
            rotation,
            health_bp: 10_000,
            powered: true,
            setting_bp: 10_000,
            stored_vu: 0,
            stored_fluid: None,
            power_generated: 0,
            cumulative_power: 0,
            active: false,
        });
        Ok(entity_id)
    }

    pub fn queue(
        &mut self,
        world: &SimulationWorld,
        device: DeviceId,
        anchor: CellPos,
        rotation: u8,
        budget: u32,
    ) -> Result<u32, DeviceError> {
        if self.reserved_budget + device.cost() > budget.saturating_sub(self.budget_spent) {
            return Err(DeviceError::InsufficientBudget);
        }
        validate_placement(world, &self.devices, &self.queued, device, anchor, rotation)?;
        let plan_id = self.next_plan_id;
        self.next_plan_id += 1;
        self.reserved_budget += device.cost();
        self.queued.push(QueuedPlan {
            plan_id,
            device,
            anchor,
            rotation,
            cost: device.cost(),
        });
        Ok(plan_id)
    }

    pub fn cancel_last_plan(&mut self) -> bool {
        if let Some(plan) = self.queued.pop() {
            self.reserved_budget = self.reserved_budget.saturating_sub(plan.cost);
            true
        } else {
            false
        }
    }

    pub fn commit_plan(
        &mut self,
        world: &SimulationWorld,
        budget: u32,
    ) -> Result<Vec<u32>, DeviceError> {
        let plans = self.queued.clone();
        for plan in &plans {
            if self.devices.iter().any(|placed| {
                footprints_overlap(placed.device, placed.anchor, plan.device, plan.anchor)
            }) {
                return Err(DeviceError::Occupied);
            }
        }
        let mut committed = Vec::with_capacity(plans.len());
        for plan in plans {
            committed.push(self.place(world, plan.device, plan.anchor, plan.rotation, budget)?);
        }
        self.queued.clear();
        self.reserved_budget = 0;
        Ok(committed)
    }

    pub fn remove(&mut self, entity_id: u32) -> bool {
        if let Some(index) = self
            .devices
            .iter()
            .position(|device| device.entity_id == entity_id)
        {
            let device = self.devices.remove(index);
            self.budget_spent = self.budget_spent.saturating_sub(device.device.cost());
            true
        } else {
            false
        }
    }

    pub fn set_selected_gate(&mut self, anchor: CellPos, setting_bp: u16) -> bool {
        self.set_selected_flow_for(anchor, setting_bp, &[DeviceId::Floodgate])
    }

    pub fn set_selected_flow(&mut self, anchor: CellPos, setting_bp: u16) -> bool {
        self.set_selected_flow_for(
            anchor,
            setting_bp,
            &[DeviceId::Floodgate, DeviceId::Pump, DeviceId::Reservoir],
        )
    }

    fn set_selected_flow_for(
        &mut self,
        anchor: CellPos,
        setting_bp: u16,
        allowed: &[DeviceId],
    ) -> bool {
        if let Some(gate) = self
            .devices
            .iter_mut()
            .find(|device| allowed.contains(&device.device) && device.anchor == anchor)
        {
            gate.setting_bp = setting_bp.min(10_000);
            true
        } else {
            false
        }
    }

    pub fn surface_flow_factor(&self, source: CellPos, destination: CellPos) -> u32 {
        self.devices
            .iter()
            .find_map(|device| {
                if device.device != DeviceId::Floodgate {
                    return None;
                }
                floodgate_controls_edge(device, source, destination)
                    .then_some(device.setting_bp as u32)
            })
            .unwrap_or(10_000)
    }

    pub fn controls_surface_edge(&self, source: CellPos, destination: CellPos) -> bool {
        self.devices
            .iter()
            .any(|device| floodgate_controls_edge(device, source, destination))
    }

    pub fn tick(&mut self, world: &mut SimulationWorld) {
        let topology = self.devices.clone();
        for device in &mut self.devices {
            device.active = false;
            device.power_generated = 0;
            match device.device {
                DeviceId::Channel => device.active = true,
                DeviceId::Pump => {
                    let source = world.index(device.anchor).unwrap();
                    let amount = world.cells[source]
                        .surface_volume()
                        .min(250)
                        .min(device.setting_bp as u32 * 250 / 10_000);
                    let outlet = step(device.anchor, direction(device.rotation));
                    let destination = pipe_endpoint(
                        &topology,
                        outlet,
                        direction(device.rotation),
                        Some(device.anchor),
                    )
                    .unwrap_or(outlet);
                    if amount > 0
                        && device.powered
                        && transfer_surface_to(world, device.anchor, destination, amount) > 0
                    {
                        device.active = true;
                    }
                }
                DeviceId::Reservoir => {
                    let index = world.index(device.anchor).unwrap();
                    let accepted = world.cells[index]
                        .surface_volume()
                        .min(8_000u32.saturating_sub(device.stored_vu));
                    if accepted > 0 {
                        let fluid = world.cells[index].surface.first().map(|entry| entry.fluid);
                        if let Some(fluid) = fluid {
                            remove_fluid(&mut world.cells[index].surface, fluid, accepted);
                            device.stored_vu += accepted;
                            device.stored_fluid = Some(fluid);
                            device.active = true;
                        }
                    }
                    // Release through the physical outlet when its valve is
                    // lowered. Stored contents remain visible and conserved.
                    let release = device
                        .stored_vu
                        .min((10_000u32.saturating_sub(device.setting_bp as u32)) * 800 / 10_000);
                    if release > 0 {
                        if let Some(fluid) = device.stored_fluid {
                            let outlet = footprint_outlet(device);
                            let destination = pipe_endpoint(
                                &topology,
                                outlet,
                                direction(device.rotation),
                                Some(device.anchor),
                            )
                            .unwrap_or(outlet);
                            if let Some(destination_index) = world.index(destination) {
                                let moved = world.cells[destination_index].add_surface(
                                    crate::simulation::FluidEntry::new(fluid, release),
                                );
                                device.stored_vu -= moved;
                                if device.stored_vu == 0 {
                                    device.stored_fluid = None;
                                }
                                device.active |= moved > 0;
                            }
                        }
                    }
                }
                DeviceId::Spillway => {
                    let index = world.index(device.anchor).unwrap();
                    let depth = world.cells[index].surface_volume();
                    if depth > 2_000 {
                        device.active = transfer_surface(
                            world,
                            device.anchor,
                            direction(device.rotation),
                            (depth - 2_000).min(500),
                        ) > 0;
                    }
                }
                DeviceId::FlowTurbine => {
                    let index = world.index(device.anchor).unwrap();
                    // Turbines are steam machines: standing surface water no
                    // longer creates free power. Connected pipes can deliver
                    // a steam stream from a reaction shelf to the turbine.
                    let local_steam = world.cells[index]
                        .airborne
                        .iter()
                        .find(|entry| entry.fluid == FluidId::Steam)
                        .map(|entry| entry.volume_vu)
                        .unwrap_or(0);
                    let mut pipe_sources = Vec::new();
                    for pipe in topology.iter().filter(|candidate| {
                        candidate.device == DeviceId::Pipe
                            && pipe_connected(&topology, device.anchor, candidate.anchor)
                    }) {
                        for position in [
                            pipe.anchor,
                            step(pipe.anchor, (0, -1)),
                            step(pipe.anchor, (1, 0)),
                            step(pipe.anchor, (0, 1)),
                            step(pipe.anchor, (-1, 0)),
                        ] {
                            let Some(source) = world.index(position) else {
                                continue;
                            };
                            if source != index
                                && !pipe_sources.contains(&source)
                                && world.cells[source]
                                    .airborne
                                    .iter()
                                    .find(|entry| entry.fluid == FluidId::Steam)
                                    .map(|entry| entry.volume_vu)
                                    .unwrap_or(0)
                                    > 0
                            {
                                pipe_sources.push(source);
                            }
                        }
                    }
                    let piped_steam: u32 = pipe_sources
                        .iter()
                        .map(|source| {
                            world.cells[*source]
                                .airborne
                                .iter()
                                .find(|entry| entry.fluid == FluidId::Steam)
                                .map(|entry| entry.volume_vu.min(400))
                                .unwrap_or(0)
                        })
                        .sum();
                    // One power unit requires a complete 40 vU packet. Leave
                    // smaller remnants in the world for a later tick instead
                    // of deleting them without power or condensate.
                    let available_flow = local_steam.saturating_add(piped_steam).min(400);
                    let flow = available_flow / 40 * 40;
                    let consumed_local = local_steam.min(flow);
                    remove_fluid(
                        &mut world.cells[index].airborne,
                        FluidId::Steam,
                        consumed_local,
                    );
                    let mut consumed_piped = flow - consumed_local;
                    for source in pipe_sources {
                        let amount = world.cells[source]
                            .airborne
                            .iter()
                            .find(|entry| entry.fluid == FluidId::Steam)
                            .map(|entry| entry.volume_vu.min(consumed_piped))
                            .unwrap_or(0);
                        remove_fluid(&mut world.cells[source].airborne, FluidId::Steam, amount);
                        consumed_piped -= amount;
                        if consumed_piped == 0 {
                            break;
                        }
                    }
                    device.power_generated = (flow / 40).min(10);
                    device.cumulative_power = device
                        .cumulative_power
                        .saturating_add(device.power_generated as u64);
                    device.active = device.power_generated > 0;
                    if device.active {
                        let condensate_target = topology
                            .iter()
                            .find(|candidate| {
                                candidate.device == DeviceId::RuneRelay
                                    && pipe_connected(&topology, device.anchor, candidate.anchor)
                            })
                            .map(|relay| relay.anchor)
                            .unwrap_or(device.anchor);
                        if let Some(target) = world.index(condensate_target) {
                            let condensed = world.cells[target].add_surface(
                                crate::simulation::FluidEntry::new(FluidId::Water, flow),
                            );
                            if condensed < flow {
                                world.cells[index].add_airborne(
                                    crate::simulation::FluidEntry::new(
                                        FluidId::Steam,
                                        flow - condensed,
                                    ),
                                );
                            }
                        }
                    }
                }
                DeviceId::Sensor => {
                    let index = world.index(device.anchor).unwrap();
                    device.active = world.cells[index].surface_volume() >= 500;
                }
                DeviceId::Filter => {
                    let index = world.index(device.anchor).unwrap();
                    if let Some(material) = world.cells[index]
                        .surface
                        .iter_mut()
                        .find(|material| material.fluid == FluidId::ToxicSlurry)
                    {
                        material.contamination_bp = material.contamination_bp.saturating_sub(2_500);
                        device.active = true;
                    }
                }
                DeviceId::RuneRelay => {
                    let index = world.index(device.anchor).unwrap();
                    let intake = world.cells[index]
                        .surface
                        .iter()
                        .find(|entry| entry.fluid == FluidId::Water)
                        .map(|entry| entry.volume_vu)
                        .unwrap_or(0)
                        .min(400)
                        .min(2_000u32.saturating_sub(device.stored_vu));
                    if intake > 0 {
                        remove_fluid(&mut world.cells[index].surface, FluidId::Water, intake);
                        device.stored_vu += intake;
                        device.stored_fluid = Some(FluidId::Water);
                    }
                    device.active = device.powered && device.stored_vu >= 100;
                }
                DeviceId::Pipe | DeviceId::Floodgate => {}
            }
        }
        // Pipe trunks also carry the compact slice's power signal.  A relay
        // cannot wake simply because it sits in water; it must be connected to
        // an operating steam turbine through the authored/placed topology.
        let powered_topology = self.devices.clone();
        for relay in self
            .devices
            .iter_mut()
            .filter(|device| device.device == DeviceId::RuneRelay)
        {
            relay.powered = powered_topology.iter().any(|turbine| {
                turbine.device == DeviceId::FlowTurbine
                    && (turbine.active || turbine.cumulative_power >= 40)
                    && pipe_connected(&powered_topology, relay.anchor, turbine.anchor)
            });
            relay.active = relay.powered && relay.stored_vu >= 100;
        }
    }
}

fn floodgate_controls_edge(device: &DeviceState, source: CellPos, destination: CellPos) -> bool {
    if device.device != DeviceId::Floodgate {
        return false;
    }
    let delta = (
        destination.x as i16 - source.x as i16,
        destination.y as i16 - source.y as i16,
    );
    let direction = direction(device.rotation);
    let source_side = source == device.anchor || destination == device.anchor;
    let matches_edge = delta == direction || delta == (-direction.0, -direction.1);
    source_side && matches_edge
}

fn remove_fluid(entries: &mut Vec<crate::simulation::FluidEntry>, fluid: FluidId, amount: u32) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.fluid == fluid) {
        entry.volume_vu -= amount.min(entry.volume_vu);
    }
    entries.retain(|entry| entry.volume_vu > 0);
}

fn validate_placement(
    world: &SimulationWorld,
    devices: &[DeviceState],
    queued: &[QueuedPlan],
    device: DeviceId,
    anchor: CellPos,
    rotation: u8,
) -> Result<(), DeviceError> {
    if rotation > 3 {
        return Err(DeviceError::InvalidRotation);
    }
    let (width, height) = device.footprint();
    for dy in 0..height {
        for dx in 0..width {
            let pos = CellPos {
                x: anchor.x + dx,
                y: anchor.y + dy,
            };
            let index = world.index(pos).ok_or(DeviceError::OutOfBounds)?;
            if world.definitions[index].protected {
                return Err(DeviceError::Protected);
            }
            if devices
                .iter()
                .any(|placed| footprint_contains(placed.device, placed.anchor, pos))
                || queued
                    .iter()
                    .any(|plan| footprint_contains(plan.device, plan.anchor, pos))
            {
                return Err(DeviceError::Occupied);
            }
        }
    }
    Ok(())
}
fn footprint_contains(device: DeviceId, anchor: CellPos, pos: CellPos) -> bool {
    let (width, height) = device.footprint();
    pos.x >= anchor.x && pos.y >= anchor.y && pos.x < anchor.x + width && pos.y < anchor.y + height
}
fn footprints_overlap(
    first: DeviceId,
    first_anchor: CellPos,
    second: DeviceId,
    second_anchor: CellPos,
) -> bool {
    let (width, height) = first.footprint();
    (0..height).any(|dy| {
        (0..width).any(|dx| {
            footprint_contains(
                second,
                second_anchor,
                CellPos {
                    x: first_anchor.x + dx,
                    y: first_anchor.y + dy,
                },
            )
        })
    })
}

#[cfg(test)]
mod tests;
