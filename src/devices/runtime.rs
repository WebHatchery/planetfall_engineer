//! Per-tick device behavior, kept separate from registry and placement rules.

use super::{
    network::{
        direction, footprint_outlet, pipe_connected, pipe_endpoint, step, transfer_surface,
        transfer_surface_to,
    },
    DeviceId, DeviceState, DeviceSystem,
};
use crate::simulation::{FluidEntry, FluidId, SimulationWorld};

impl DeviceSystem {
    pub fn tick(&mut self, world: &mut SimulationWorld) {
        let topology = self.devices.clone();
        for device in &mut self.devices {
            device.active = false;
            device.power_generated = 0;
            match device.device {
                DeviceId::Channel => device.active = true,
                DeviceId::Pump => tick_pump(device, world, &topology),
                DeviceId::Reservoir => tick_reservoir(device, world, &topology),
                DeviceId::Spillway => tick_spillway(device, world),
                DeviceId::FlowTurbine => tick_turbine(device, world, &topology),
                DeviceId::Sensor => tick_sensor(device, world),
                DeviceId::Filter => tick_filter(device, world),
                DeviceId::RuneRelay => tick_relay(device, world),
                DeviceId::Pipe | DeviceId::Floodgate => {}
            }
            if device.active {
                device.operated_ticks = device.operated_ticks.saturating_add(1);
            }
        }
        let powered_topology = self.devices.clone();
        for relay in self
            .devices
            .iter_mut()
            .filter(|device| device.device == DeviceId::RuneRelay)
        {
            let turbine_connected = powered_topology.iter().any(|turbine| {
                turbine.device == DeviceId::FlowTurbine
                    && (turbine.active || turbine.cumulative_power >= 40)
                    && pipe_connected(&powered_topology, relay.anchor, turbine.anchor)
            });
            relay.active =
                relay.powered && relay.enabled && relay.stored_vu >= 100 && turbine_connected;
        }
    }
}

fn tick_pump(device: &mut DeviceState, world: &mut SimulationWorld, topology: &[DeviceState]) {
    if !device.enabled || !device.powered {
        return;
    }
    let Some(source) = world.index(device.anchor) else {
        return;
    };
    let amount = world.cells[source]
        .surface_volume()
        .min(250)
        .min(device.setting_bp as u32 * 250 / 10_000);
    let outlet = step(device.anchor, direction(device.rotation));
    let destination = pipe_endpoint(
        topology,
        outlet,
        direction(device.rotation),
        Some(device.anchor),
    )
    .unwrap_or(outlet);
    device.active =
        amount > 0 && transfer_surface_to(world, device.anchor, destination, amount) > 0;
}

fn tick_reservoir(device: &mut DeviceState, world: &mut SimulationWorld, topology: &[DeviceState]) {
    if !device.enabled || !device.powered {
        return;
    }
    let Some(index) = world.index(device.anchor) else {
        return;
    };
    let accepted = world.cells[index]
        .surface_volume()
        .min(8_000u32.saturating_sub(device.stored_vu));
    if accepted > 0 {
        if let Some(fluid) = world.cells[index].surface.first().map(|entry| entry.fluid) {
            remove_fluid(&mut world.cells[index].surface, fluid, accepted);
            device.stored_vu += accepted;
            device.stored_fluid = Some(fluid);
            device.active = true;
        }
    }
    let release = device
        .stored_vu
        .min((10_000u32.saturating_sub(device.setting_bp as u32)) * 800 / 10_000);
    if release == 0 {
        return;
    }
    let Some(fluid) = device.stored_fluid else {
        return;
    };
    let outlet = footprint_outlet(device);
    let destination = pipe_endpoint(
        topology,
        outlet,
        direction(device.rotation),
        Some(device.anchor),
    )
    .unwrap_or(outlet);
    if let Some(destination_index) = world.index(destination) {
        let moved = world.cells[destination_index].add_surface(FluidEntry::new(fluid, release));
        device.stored_vu -= moved;
        if device.stored_vu == 0 {
            device.stored_fluid = None;
        }
        device.active |= moved > 0;
    }
}

fn tick_spillway(device: &mut DeviceState, world: &mut SimulationWorld) {
    let Some(index) = world.index(device.anchor) else {
        return;
    };
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

fn tick_turbine(device: &mut DeviceState, world: &mut SimulationWorld, topology: &[DeviceState]) {
    let Some(index) = world.index(device.anchor) else {
        return;
    };
    let local_steam = material_volume(&world.cells[index].airborne, FluidId::Steam);
    let local_water = material_volume(&world.cells[index].surface, FluidId::Water);
    let mut pipe_sources = Vec::new();
    for pipe in topology.iter().filter(|candidate| {
        candidate.device == DeviceId::Pipe
            && pipe_connected(topology, device.anchor, candidate.anchor)
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
            let has_steam = material_volume(&world.cells[source].airborne, FluidId::Steam) > 0;
            let has_water = material_volume(&world.cells[source].surface, FluidId::Water) > 0;
            if source != index && !pipe_sources.contains(&source) && (has_steam || has_water) {
                pipe_sources.push(source);
            }
        }
    }
    let turbine_fluid = if local_steam > 0
        || world
            .cells
            .iter()
            .any(|cell| material_volume(&cell.airborne, FluidId::Steam) > 0)
    {
        FluidId::Steam
    } else {
        FluidId::Water
    };
    let local_flow = if turbine_fluid == FluidId::Steam {
        local_steam
    } else {
        local_water
    };
    let piped_flow: u32 = pipe_sources
        .iter()
        .map(|source| material_volume_for_turbine(&world.cells[*source], turbine_fluid).min(400))
        .sum();
    let available_flow = local_flow.saturating_add(piped_flow).min(400);
    let flow = available_flow / 40 * 40;
    device.power_generated = (flow / 100).min(4);
    if device.power_generated == 0 {
        return;
    }
    remove_turbine_material(&mut world.cells[index], turbine_fluid, local_flow.min(flow));
    let mut consumed_piped = flow.saturating_sub(local_flow);
    for source in pipe_sources {
        let amount =
            material_volume_for_turbine(&world.cells[source], turbine_fluid).min(consumed_piped);
        remove_turbine_material(&mut world.cells[source], turbine_fluid, amount);
        consumed_piped -= amount;
        if consumed_piped == 0 {
            break;
        }
    }
    device.cumulative_power = device
        .cumulative_power
        .saturating_add(device.power_generated as u64);
    device.active = device.power_generated > 0;
    if device.active {
        let target = topology
            .iter()
            .find(|candidate| {
                candidate.device == DeviceId::RuneRelay
                    && pipe_connected(topology, device.anchor, candidate.anchor)
            })
            .map(|relay| relay.anchor)
            .unwrap_or(device.anchor);
        if let Some(target_index) = world.index(target) {
            let condensed =
                world.cells[target_index].add_surface(FluidEntry::new(FluidId::Water, flow));
            if condensed < flow {
                let remainder = flow - condensed;
                if turbine_fluid == FluidId::Steam {
                    world.cells[index].add_airborne(FluidEntry::new(FluidId::Steam, remainder));
                } else {
                    world.cells[index].add_surface(FluidEntry::new(FluidId::Water, remainder));
                }
            }
        }
    }
}

fn material_volume(entries: &[FluidEntry], fluid: FluidId) -> u32 {
    entries
        .iter()
        .find(|entry| entry.fluid == fluid)
        .map(|entry| entry.volume_vu)
        .unwrap_or(0)
}

fn material_volume_for_turbine(cell: &crate::simulation::SimCell, fluid: FluidId) -> u32 {
    if fluid == FluidId::Steam {
        material_volume(&cell.airborne, fluid)
    } else {
        material_volume(&cell.surface, fluid)
    }
}

fn remove_turbine_material(cell: &mut crate::simulation::SimCell, fluid: FluidId, amount: u32) {
    if fluid == FluidId::Steam {
        remove_fluid(&mut cell.airborne, fluid, amount);
    } else {
        remove_fluid(&mut cell.surface, fluid, amount);
    }
}

fn tick_sensor(device: &mut DeviceState, world: &mut SimulationWorld) {
    if device.enabled && device.powered {
        if let Some(index) = world.index(device.anchor) {
            device.active = world.cells[index].surface_volume() >= 500;
        }
    }
}

fn tick_filter(device: &mut DeviceState, world: &mut SimulationWorld) {
    if !device.enabled || !device.powered {
        return;
    }
    if let Some(index) = world.index(device.anchor) {
        if let Some(material) = world.cells[index]
            .surface
            .iter_mut()
            .find(|material| material.fluid == FluidId::ToxicSlurry)
        {
            material.contamination_bp = material.contamination_bp.saturating_sub(2_500);
            device.active = true;
        }
    }
}

fn tick_relay(device: &mut DeviceState, world: &mut SimulationWorld) {
    if !device.enabled || !device.powered {
        return;
    }
    let Some(index) = world.index(device.anchor) else {
        return;
    };
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
    device.active = device.stored_vu >= 100;
}

fn remove_fluid(entries: &mut Vec<FluidEntry>, fluid: FluidId, amount: u32) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.fluid == fluid) {
        entry.volume_vu -= amount.min(entry.volume_vu);
    }
    entries.retain(|entry| entry.volume_vu > 0);
}
