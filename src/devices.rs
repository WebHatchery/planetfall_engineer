//! Deterministic slice device registry, placement rules, and showcase fixtures.

use crate::{
    economy::{FabricationState, PowerClass, PowerLedger},
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};
use serde::{Deserialize, Serialize};

mod network;
mod runtime;
mod showcase;
use network::direction;
pub use network::{pipe_connected, pipe_endpoint};
pub use showcase::{run_all_showcases, run_showcase, showcase_world};

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

    pub const fn power_demand_eu(self) -> u32 {
        match self {
            Self::Pump | Self::Filter => 3,
            Self::Reservoir | Self::Sensor => 1,
            Self::RuneRelay => 2,
            _ => 0,
        }
    }

    pub const fn power_class(self) -> Option<PowerClass> {
        match self {
            Self::Reservoir | Self::Sensor => Some(PowerClass::Safety),
            Self::Pump => Some(PowerClass::Transport),
            Self::Filter => Some(PowerClass::Process),
            Self::RuneRelay => Some(PowerClass::Interface),
            _ => None,
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
    InsufficientFabrication,
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
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub operated_ticks: u64,
    #[serde(default)]
    pub authored: bool,
}

const fn default_enabled() -> bool {
    true
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
    pub next_plan_id: u32,
    pub queued: Vec<QueuedPlan>,
    pub devices: Vec<DeviceState>,
}

impl DeviceSystem {
    /// Install map-authored infrastructure without charging player fabrication.
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
            enabled: true,
            operated_ticks: 0,
            authored: true,
        });
        Ok(entity_id)
    }

    pub fn place(
        &mut self,
        world: &SimulationWorld,
        device: DeviceId,
        anchor: CellPos,
        rotation: u8,
        fabrication: &mut FabricationState,
    ) -> Result<u32, DeviceError> {
        if rotation > 3 {
            return Err(DeviceError::InvalidRotation);
        }
        validate_placement(world, &self.devices, &[], device, anchor, rotation)?;
        fabrication
            .spend_immediately(device.cost())
            .map_err(|_| DeviceError::InsufficientFabrication)?;
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
            enabled: true,
            operated_ticks: 0,
            authored: false,
        });
        Ok(entity_id)
    }

    pub fn queue(
        &mut self,
        world: &SimulationWorld,
        device: DeviceId,
        anchor: CellPos,
        rotation: u8,
        fabrication: &mut FabricationState,
    ) -> Result<u32, DeviceError> {
        validate_placement(world, &self.devices, &self.queued, device, anchor, rotation)?;
        fabrication
            .reserve(device.cost())
            .map_err(|_| DeviceError::InsufficientFabrication)?;
        let plan_id = self.next_plan_id;
        self.next_plan_id += 1;
        self.queued.push(QueuedPlan {
            plan_id,
            device,
            anchor,
            rotation,
            cost: device.cost(),
        });
        Ok(plan_id)
    }

    pub fn cancel_last_plan(&mut self, fabrication: &mut FabricationState) -> bool {
        if let Some(plan) = self.queued.pop() {
            fabrication.release_reservation(plan.cost);
            true
        } else {
            false
        }
    }

    pub fn commit_plan(
        &mut self,
        world: &SimulationWorld,
        fabrication: &mut FabricationState,
    ) -> Result<Vec<u32>, DeviceError> {
        let plans = self.queued.clone();
        for plan in &plans {
            validate_placement(
                world,
                &self.devices,
                &[],
                plan.device,
                plan.anchor,
                plan.rotation,
            )?;
            if self.devices.iter().any(|placed| {
                footprints_overlap(placed.device, placed.anchor, plan.device, plan.anchor)
            }) {
                return Err(DeviceError::Occupied);
            }
        }
        let total_cost: u32 = plans.iter().map(|plan| plan.cost).sum();
        fabrication
            .spend_reservation(total_cost)
            .map_err(|_| DeviceError::InsufficientFabrication)?;
        let mut committed = Vec::with_capacity(plans.len());
        for plan in plans {
            let entity_id = self.next_entity_id;
            self.next_entity_id += 1;
            self.devices.push(new_player_device(entity_id, &plan));
            committed.push(entity_id);
        }
        self.queued.clear();
        Ok(committed)
    }

    pub fn remove(&mut self, entity_id: u32, fabrication: &mut FabricationState) -> bool {
        if let Some(index) = self
            .devices
            .iter()
            .position(|device| device.entity_id == entity_id)
        {
            let device = self.devices.remove(index);
            if !device.authored {
                let refund = if device.operated_ticks == 0 {
                    device.device.cost()
                } else {
                    device.device.cost() * 3 / 4
                };
                fabrication.refund(refund);
            }
            true
        } else {
            false
        }
    }

    pub fn set_selected_gate(&mut self, anchor: CellPos, setting_bp: u16) -> bool {
        self.set_selected_flow_for(anchor, setting_bp, &[DeviceId::Floodgate])
    }

    pub fn set_enabled(&mut self, entity_id: u32, enabled: bool) -> bool {
        if let Some(device) = self
            .devices
            .iter_mut()
            .find(|device| device.entity_id == entity_id)
        {
            device.enabled = enabled;
            if !enabled {
                device.powered = false;
                device.active = false;
            }
            true
        } else {
            false
        }
    }

    pub fn allocate_power(
        &mut self,
        ledger: &mut PowerLedger,
        authored_supply_eu: u32,
    ) -> Vec<(u32, u32)> {
        let turbine_supply_eu: u32 = self
            .devices
            .iter()
            .filter(|device| device.device == DeviceId::FlowTurbine)
            .map(|device| device.power_generated)
            .sum();
        ledger.reset(authored_supply_eu, turbine_supply_eu);
        let supply = ledger.supply_eu();
        let mut indices: Vec<usize> = self
            .devices
            .iter()
            .enumerate()
            .filter(|(_, device)| device.enabled && device.device.power_demand_eu() > 0)
            .map(|(index, _)| index)
            .collect();
        indices.sort_by_key(|index| {
            (
                self.devices[*index]
                    .device
                    .power_class()
                    .unwrap_or(PowerClass::Interface),
                self.devices[*index].entity_id,
            )
        });
        let total_demand: u32 = indices
            .iter()
            .map(|index| self.devices[*index].device.power_demand_eu())
            .sum();
        let mut remaining = supply;
        let mut brownouts = Vec::new();
        for index in indices {
            let device = &mut self.devices[index];
            let demand = device.device.power_demand_eu();
            let was_powered = device.powered;
            device.powered = remaining >= demand;
            if device.powered {
                remaining -= demand;
                ledger.allocated_demand_eu += demand;
            } else if was_powered {
                brownouts.push((device.entity_id, demand));
            }
        }
        for device in &mut self.devices {
            if device.device.power_demand_eu() == 0 {
                device.powered = true;
            }
        }
        ledger.curtailed_supply_eu = remaining;
        ledger.deficit_eu = total_demand.saturating_sub(supply);
        brownouts
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
}

fn new_player_device(entity_id: u32, plan: &QueuedPlan) -> DeviceState {
    DeviceState {
        entity_id,
        device: plan.device,
        anchor: plan.anchor,
        rotation: plan.rotation,
        health_bp: 10_000,
        powered: true,
        setting_bp: 10_000,
        stored_vu: 0,
        stored_fluid: None,
        power_generated: 0,
        cumulative_power: 0,
        active: false,
        enabled: true,
        operated_ticks: 0,
        authored: false,
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
