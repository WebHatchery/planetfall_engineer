//! Authored campaign map constructors and reference fixtures.

use crate::{
    devices::DeviceId,
    mission::MissionId,
    simulation::{CellDefinition, FluidId, SimulationWorld},
    state::CellPos,
};

#[derive(Debug, Clone)]
pub struct CampaignMap {
    pub world: SimulationWorld,
    pub budget: u32,
    pub reference_tick_range: (u64, u64),
}

pub fn load_campaign(id: MissionId) -> CampaignMap {
    let (width, height) = id.map_size();
    let budget = match id {
        MissionId::L01FirstFlow => 40,
        MissionId::L02HoldingLine => 105,
        MissionId::L03Firebreak => 160,
    };
    let reference_tick_range = match id {
        MissionId::L01FirstFlow => (900, 1_400),
        MissionId::L02HoldingLine => (1_400, 1_800),
        MissionId::L03Firebreak => (1_500, 2_100),
    };
    let mut world = SimulationWorld::new(width, height);
    match id {
        MissionId::L01FirstFlow => author_l01(&mut world),
        MissionId::L02HoldingLine => author_l02(&mut world),
        MissionId::L03Firebreak => author_l03(&mut world),
    }
    CampaignMap {
        world,
        budget,
        reference_tick_range,
    }
}

fn author_l01(world: &mut SimulationWorld) {
    set_ambient(world, 3_030);
    // A west-to-east meltwater course crosses three southward runoff cuts.
    // Each cut is lower than the onward course and ends in a fissure, so an
    // untouched simulation loses every drop. Raising one cell at each cut
    // forms three small barriers and sends the stream onward to the dam.
    for y in 0..world.height {
        for x in 0..world.width {
            set_height(world, CellPos { x, y }, 2_400);
        }
    }
    for x in 2..=25 {
        set_height(world, CellPos { x, y: 9 }, 1_900 - (x as i16 * 25));
    }
    for (x, drain_y) in [(8, 14), (14, 15), (20, 14)] {
        let course_height = 1_900 - (x as i16 * 25);
        set_height(world, CellPos { x: x + 1, y: 9 }, course_height + 100);
        for y in 10..=drain_y {
            set_height(
                world,
                CellPos { x, y },
                course_height - 100 - (y as i16 - 10) * 100,
            );
        }
        let drain = CellPos { x, y: drain_y };
        set_surface_drain(world, drain, 800);
        protect(world, drain);
    }
    for y in 7..=11 {
        for x in 26..=30 {
            set_height(world, CellPos { x, y }, 600);
            set_sealed(world, CellPos { x, y });
            set_contained(world, CellPos { x, y });
        }
    }
    world.add_source(CellPos { x: 2, y: 9 }, FluidId::Water, 20);
    protect(world, CellPos { x: 24, y: 8 });
    // The dam inlet is already open; solving the mission is about shaping the
    // watershed, not operating a switch after pressing play.
    install_device(
        world,
        DeviceId::Floodgate,
        CellPos { x: 25, y: 9 },
        0,
        10_000,
    );
}

fn author_l02(world: &mut SimulationWorld) {
    set_ambient(world, 2_930);
    world.add_source(CellPos { x: 5, y: 16 }, FluidId::Water, 100);
    for y in 0..world.height {
        for x in 0..world.width {
            let height = if y < 4 {
                1_750
            } else if x > 29 {
                1_250
            } else if x > 10 && y < 14 {
                1_100
            } else {
                850
            };
            set_height(world, CellPos { x, y }, height);
        }
    }
    // Low aquifer and raised central reserve make pumping legible at a glance.
    for y in 13..=19 {
        for x in 2..=8 {
            set_height(world, CellPos { x, y }, 650);
        }
    }
    for y in 14..=18 {
        for x in 3..=7 {
            set_height(world, CellPos { x, y }, 0);
            set_sealed(world, CellPos { x, y });
        }
    }
    world.inject(CellPos { x: 5, y: 16 }, FluidId::Water, 8_000);
    for y in 5..=11 {
        for x in 17..=23 {
            set_height(world, CellPos { x, y }, 2_000);
        }
    }
    for y in 7..=8 {
        for x in 19..=20 {
            set_sealed(world, CellPos { x, y });
        }
    }
    for y in 14..=18 {
        for x in 24..=28 {
            set_height(world, CellPos { x, y }, 5_000);
            protect(world, CellPos { x, y });
        }
    }
    for y in 7..=9 {
        for x in 31..=36 {
            set_height(world, CellPos { x, y }, 0);
            set_sealed(world, CellPos { x, y });
            set_contained(world, CellPos { x, y });
        }
    }
    // Two pipe runs are intentionally broken into twelve obvious gaps. They
    // read as infrastructure rather than a generic empty board and leave the
    // player with meaningful connection work.
    for (x, y, rotation) in [
        (6, 16, 0),
        (7, 16, 0),
        (8, 16, 0),
        (10, 16, 0),
        (12, 16, 0),
        (14, 16, 0),
        (16, 16, 1),
        (16, 15, 1),
        (16, 14, 0),
        (17, 14, 1),
        (17, 13, 0),
        (18, 13, 1),
        (18, 12, 0),
        (22, 8, 0),
        (24, 8, 0),
        (26, 8, 0),
        (28, 8, 0),
        (30, 8, 0),
    ] {
        install_device(world, DeviceId::Pipe, CellPos { x, y }, rotation, 0);
    }
    set_height(world, CellPos { x: 39, y: 18 }, 0);
}

fn author_l03(world: &mut SimulationWorld) {
    set_ambient(world, 3_230);
    world.add_source(CellPos { x: 5, y: 15 }, FluidId::Lava, 100);
    // Caldera walls and a descending lava trough frame the reaction shelf.
    for y in 0..world.height {
        for x in 0..world.width {
            let rim = x.min(y).min(world.width - 1 - x).min(world.height - 1 - y);
            let inner_noise = ((u32::from(x) * 17 + u32::from(y) * 31) % 5) as i16 * 70;
            let height = if rim < 3 {
                2_600
            } else if rim < 6 {
                1_800 + inner_noise
            } else {
                900 + inner_noise
            };
            set_height(world, CellPos { x, y }, height);
        }
    }
    for x in 5..=35 {
        set_height(world, CellPos { x, y: 15 }, 350);
        if x % 3 != 0 {
            set_height(world, CellPos { x, y: 14 }, 650);
        }
    }
    // Geothermal ground keeps the authored approach molten long enough to
    // reach the reaction shelf; the shelf itself remains cool so water contact
    // produces steam and durable basalt there.
    for x in 5..=19 {
        set_cell_ambient(world, CellPos { x, y: 15 }, 10_000);
        set_contained(world, CellPos { x, y: 15 });
    }
    for y in 3..=6 {
        for x in 8..=11 {
            set_height(world, CellPos { x, y }, 250);
        }
    }
    for y in 3..=6 {
        for x in 8..=11 {
            world.inject(CellPos { x, y }, FluidId::Water, 750);
        }
    }
    for y in 12..=18 {
        for x in 35..=40 {
            protect(world, CellPos { x, y });
        }
    }
    for y in 12..=16 {
        for x in 20..=25 {
            set_sealed(world, CellPos { x, y });
            set_height(world, CellPos { x, y }, 500);
        }
    }
    set_sealed(world, CellPos { x: 27, y: 9 });
    for y in 5..=6 {
        for x in 38..=39 {
            set_height(world, CellPos { x, y }, 2_000);
        }
    }
    for x in 24..=30 {
        set_height(world, CellPos { x, y: 9 }, 300);
    }
    // Lava enters the sealed shelf through a real headworks gate. Water must
    // arrive independently from the finite cistern through the service line.
    install_device(
        world,
        DeviceId::Floodgate,
        CellPos { x: 19, y: 15 },
        0,
        10_000,
    );
    for (x, y, rotation) in [
        (12, 5, 0),
        (13, 5, 0),
        (14, 5, 0),
        (16, 5, 0),
        (18, 5, 0),
        (20, 5, 1),
        (20, 7, 1),
        (20, 9, 1),
        (20, 11, 1),
        (20, 13, 1),
        (20, 15, 0),
        (21, 15, 0),
        (22, 15, 0),
    ] {
        install_device(world, DeviceId::Pipe, CellPos { x, y }, rotation, 0);
    }
    install_device(world, DeviceId::Channel, CellPos { x: 30, y: 9 }, 0, 0);
}

pub fn apply_scheduled_events(world: &mut SimulationWorld, id: MissionId) {
    let (start_tick, end_tick, surge_rate) = match id {
        MissionId::L02HoldingLine => (900, 1_200, 400),
        MissionId::L03Firebreak => (1_000, 1_250, 220),
        MissionId::L01FirstFlow => return,
    };
    if world.tick == start_tick {
        if let Some(source) = world.sources.first_mut() {
            source.rate_vu = surge_rate;
        }
    } else if world.tick == end_tick {
        if let Some(source) = world.sources.first_mut() {
            source.rate_vu = 100;
        }
    }
}

fn cell_mut(world: &mut SimulationWorld, pos: CellPos) -> &mut crate::simulation::SimCell {
    let index = world
        .index(pos)
        .expect("authored campaign anchor is in bounds");
    &mut world.cells[index]
}
fn set_height(world: &mut SimulationWorld, pos: CellPos, height: i16) {
    cell_mut(world, pos).height_hu = height;
}
fn set_sealed(world: &mut SimulationWorld, pos: CellPos) {
    cell_mut(world, pos).sealed = true;
}
fn set_contained(world: &mut SimulationWorld, pos: CellPos) {
    cell_mut(world, pos).contained = true;
}
fn protect(world: &mut SimulationWorld, pos: CellPos) {
    let index = world
        .index(pos)
        .expect("authored protected anchor is in bounds");
    world.definitions[index].protected = true;
}
fn set_surface_drain(world: &mut SimulationWorld, pos: CellPos, rate_vu: u32) {
    let index = world
        .index(pos)
        .expect("authored drain anchor is in bounds");
    world.definitions[index].surface_drain_rate_vu = rate_vu;
}
fn install_device(
    world: &mut SimulationWorld,
    device: DeviceId,
    anchor: CellPos,
    rotation: u8,
    setting_bp: u16,
) {
    let mut devices = std::mem::take(&mut world.devices);
    let placed = devices.install_fixture(world, device, anchor, rotation);
    if let Ok(entity) = placed {
        if let Some(state) = devices
            .devices
            .iter_mut()
            .find(|state| state.entity_id == entity)
        {
            state.setting_bp = setting_bp;
        }
    }
    world.devices = devices;
}
fn set_ambient(world: &mut SimulationWorld, ambient: i32) {
    for definition in &mut world.definitions {
        *definition = CellDefinition {
            ambient_temperature_dk: ambient,
            ..definition.clone()
        };
    }
}

fn set_cell_ambient(world: &mut SimulationWorld, pos: CellPos, ambient: i32) {
    let index = world
        .index(pos)
        .expect("authored campaign anchor is in bounds");
    world.definitions[index].ambient_temperature_dk = ambient;
}

#[cfg(test)]
mod tests;
