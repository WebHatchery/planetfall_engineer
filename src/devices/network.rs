//! Deterministic conduit traversal and material delivery helpers.

use super::{remove_fluid, DeviceId, DeviceState};
use crate::{simulation::SimulationWorld, state::CellPos};

pub(super) fn direction(rotation: u8) -> (i16, i16) {
    match rotation % 4 {
        0 => (1, 0),
        1 => (0, 1),
        2 => (-1, 0),
        _ => (0, -1),
    }
}

pub(super) fn step(source: CellPos, (dx, dy): (i16, i16)) -> CellPos {
    CellPos {
        x: source.x.saturating_add_signed(dx),
        y: source.y.saturating_add_signed(dy),
    }
}

fn adjacent(a: CellPos, b: CellPos) -> bool {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y) == 1
}

fn adjacent_to_footprint(position: CellPos, device: &DeviceState) -> bool {
    let (width, height) = device.device.footprint();
    (0..height).any(|dy| {
        (0..width).any(|dx| {
            adjacent(
                position,
                CellPos {
                    x: device.anchor.x + dx,
                    y: device.anchor.y + dy,
                },
            )
        })
    })
}

fn is_conduit(device: DeviceId) -> bool {
    matches!(device, DeviceId::Pipe | DeviceId::Channel)
}

pub(super) fn pipe_connected(devices: &[DeviceState], first: CellPos, second: CellPos) -> bool {
    let mut frontier = vec![first];
    let mut visited = Vec::new();
    while let Some(position) = frontier.pop() {
        if visited.contains(&position) {
            continue;
        }
        visited.push(position);
        if adjacent(position, second) {
            return true;
        }
        for pipe in devices.iter().filter(|device| is_conduit(device.device)) {
            if adjacent(position, pipe.anchor) && !visited.contains(&pipe.anchor) {
                frontier.push(pipe.anchor);
            }
        }
    }
    false
}

pub(super) fn pipe_endpoint(
    devices: &[DeviceState],
    start: CellPos,
    direction: (i16, i16),
    excluded_anchor: Option<CellPos>,
) -> Option<CellPos> {
    if !devices
        .iter()
        .any(|device| is_conduit(device.device) && device.anchor == start)
    {
        return None;
    }
    let mut frontier = vec![(start, None, 0u32)];
    let mut visited = Vec::new();
    let mut farthest = (start, None, 0u32);
    while let Some((position, previous, distance)) = frontier.pop() {
        if visited.contains(&position) {
            continue;
        }
        visited.push(position);
        if distance > farthest.2 {
            farthest = (position, previous, distance);
        }
        let endpoint = devices
            .iter()
            .filter(|device| {
                !is_conduit(device.device)
                    && device.device != DeviceId::Pump
                    && Some(device.anchor) != excluded_anchor
                    && adjacent_to_footprint(position, device)
            })
            .min_by_key(|device| device.entity_id);
        if let Some(endpoint) = endpoint {
            return Some(endpoint.anchor);
        }
        for pipe in devices
            .iter()
            .filter(|device| is_conduit(device.device) && adjacent(position, device.anchor))
        {
            frontier.push((pipe.anchor, Some(position), distance + 1));
        }
    }
    let exit_direction = farthest
        .1
        .map(|previous| {
            (
                farthest.0.x as i16 - previous.x as i16,
                farthest.0.y as i16 - previous.y as i16,
            )
        })
        .unwrap_or(direction);
    Some(step(farthest.0, exit_direction))
}

pub(super) fn footprint_outlet(device: &DeviceState) -> CellPos {
    let (width, height) = device.device.footprint();
    match device.rotation % 4 {
        0 => CellPos {
            x: device.anchor.x.saturating_add(width),
            y: device.anchor.y,
        },
        1 => CellPos {
            x: device.anchor.x,
            y: device.anchor.y.saturating_add(height),
        },
        2 => step(device.anchor, (-1, 0)),
        _ => step(device.anchor, (0, -1)),
    }
}

pub(super) fn transfer_surface(
    world: &mut SimulationWorld,
    source: CellPos,
    (dx, dy): (i16, i16),
    amount: u32,
) -> u32 {
    let x = source.x as i16 + dx;
    let y = source.y as i16 + dy;
    if x < 0 || y < 0 {
        return 0;
    }
    transfer_surface_to(
        world,
        source,
        CellPos {
            x: x as u16,
            y: y as u16,
        },
        amount,
    )
}

pub(super) fn transfer_surface_to(
    world: &mut SimulationWorld,
    source: CellPos,
    destination: CellPos,
    amount: u32,
) -> u32 {
    let Some(source_index) = world.index(source) else {
        return 0;
    };
    let Some(destination_index) = world.index(destination) else {
        return 0;
    };
    let Some(entry) = world.cells[source_index].surface.first().cloned() else {
        return 0;
    };
    let moved = amount.min(entry.volume_vu).min(
        crate::simulation::CELL_CAPACITY_VU
            .saturating_sub(world.cells[destination_index].surface_volume()),
    );
    if moved == 0 {
        return 0;
    }
    remove_fluid(&mut world.cells[source_index].surface, entry.fluid, moved);
    world.cells[destination_index].add_surface(crate::simulation::FluidEntry {
        fluid: entry.fluid,
        volume_vu: moved,
        temperature_dk: entry.temperature_dk,
        contamination_bp: entry.contamination_bp,
    });
    moved
}
