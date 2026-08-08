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
        MissionId::L01FirstFlow => (480, 750),
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
    // Ashfall Basin is intentionally sculpted as a small, legible opening
    // puzzle: high meltwater terrace -> winding cut -> basin, with a ridge
    // that asks the player to excavate instead of simply running time.
    for y in 0..world.height {
        for x in 0..world.width {
            let height = if y < 3 {
                1_750
            } else if x < 10 {
                1_500
            } else if x < 20 {
                1_100
            } else {
                850
            };
            set_height(world, CellPos { x, y }, height);
        }
    }
    for y in 2..=6 {
        for x in 2..=8 {
            set_height(world, CellPos { x, y }, 2_000);
        }
    }
    for y in 4..=11 {
        for x in 12..=18 {
            set_height(world, CellPos { x, y }, 1_850);
        }
    }
    for (pos, height) in [
        (CellPos { x: 4, y: 4 }, 1_750),
        (CellPos { x: 5, y: 4 }, 1_600),
        (CellPos { x: 6, y: 4 }, 1_500),
        (CellPos { x: 7, y: 5 }, 1_400),
        (CellPos { x: 8, y: 5 }, 1_300),
        (CellPos { x: 9, y: 6 }, 1_200),
        (CellPos { x: 10, y: 6 }, 1_100),
        (CellPos { x: 11, y: 7 }, 1_050),
        (CellPos { x: 12, y: 8 }, 1_000),
        (CellPos { x: 13, y: 8 }, 900),
        (CellPos { x: 14, y: 8 }, 800),
        (CellPos { x: 15, y: 8 }, 700),
        (CellPos { x: 16, y: 8 }, 600),
        (CellPos { x: 17, y: 8 }, 500),
        (CellPos { x: 18, y: 8 }, 400),
        (CellPos { x: 19, y: 8 }, 300),
        (CellPos { x: 20, y: 8 }, 250),
        (CellPos { x: 21, y: 8 }, 250),
    ] {
        set_height(world, pos, height);
    }
    for y in 5..=12 {
        for x in 20..=27 {
            set_height(world, CellPos { x, y }, 900);
        }
    }
    for y in 7..=10 {
        for x in 22..=25 {
            set_height(world, CellPos { x, y }, 0);
        }
    }
    for y in 14..=17 {
        for x in 26..=29 {
            set_height(world, CellPos { x, y }, 0);
        }
    }
    world.add_source(CellPos { x: 4, y: 4 }, FluidId::Water, 180);
    // The gate feeds a maintained basin inlet. The high terrace source remains
    // scenic runoff; this controlled source is the puzzle's reliable supply.
    world.add_source(CellPos { x: 22, y: 8 }, FluidId::Water, 10_000);
    protect(world, CellPos { x: 10, y: 6 });
    for y in 7..=10 {
        for x in 22..=25 {
            set_sealed(world, CellPos { x, y });
        }
    }
    // A real, initially closed inlet gate establishes the tutorial's first
    // machine relationship instead of asking the player to imagine one.
    install_device(world, DeviceId::Floodgate, CellPos { x: 21, y: 8 }, 0, 0);
}

fn author_l02(world: &mut SimulationWorld) {
    set_ambient(world, 2_930);
    world.add_source(CellPos { x: 5, y: 16 }, FluidId::Water, 100);
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
        }
    }
    world.add_source(CellPos { x: 33, y: 8 }, FluidId::Water, 10_000);
    // Two pipe runs are intentionally broken into twelve obvious gaps. They
    // read as infrastructure rather than a generic empty board and leave the
    // player with meaningful connection work.
    for pos in [
        (8, 16),
        (10, 16),
        (12, 16),
        (14, 16),
        (16, 14),
        (18, 12),
        (22, 9),
        (24, 9),
        (26, 9),
        (28, 9),
        (30, 9),
        (37, 8),
    ] {
        install_device(world, DeviceId::Pipe, CellPos { x: pos.0, y: pos.1 }, 0, 0);
    }
    set_height(world, CellPos { x: 39, y: 18 }, 0);
}

fn author_l03(world: &mut SimulationWorld) {
    set_ambient(world, 3_230);
    world.add_source(CellPos { x: 5, y: 15 }, FluidId::Lava, 100);
    // Paired vents at the reaction shelf make the firebreak a live system,
    // while the cistern remains the player's visible reserve.
    world.add_source(CellPos { x: 22, y: 14 }, FluidId::Water, 220);
    world.add_source(CellPos { x: 22, y: 14 }, FluidId::Lava, 220);
    // Caldera walls and a descending lava trough frame the reaction shelf.
    for y in 0..world.height {
        for x in 0..world.width {
            let rim = x.min(y).min(world.width - 1 - x).min(world.height - 1 - y);
            set_height(world, CellPos { x, y }, if rim < 3 { 2_000 } else { 900 });
        }
    }
    for x in 5..=35 {
        set_height(world, CellPos { x, y: 15 }, 350);
        if x % 3 != 0 {
            set_height(world, CellPos { x, y: 14 }, 650);
        }
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
fn protect(world: &mut SimulationWorld, pos: CellPos) {
    let index = world
        .index(pos)
        .expect("authored protected anchor is in bounds");
    world.definitions[index].protected = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_campaign_maps_have_authored_sizes_and_budget() {
        let maps: Vec<_> = MissionId::ALL.into_iter().map(load_campaign).collect();
        assert_eq!((maps[0].world.width, maps[0].world.height), (32, 20));
        assert_eq!((maps[1].world.width, maps[1].world.height), (40, 24));
        assert_eq!((maps[2].world.width, maps[2].world.height), (48, 30));
        assert_eq!(
            maps.iter().map(|map| map.budget).collect::<Vec<_>>(),
            vec![40, 105, 160]
        );
    }
    #[test]
    fn l01_has_normative_protected_beacon_and_basin() {
        let map = load_campaign(MissionId::L01FirstFlow);
        assert!(map.world.definitions[map.world.index(CellPos { x: 10, y: 6 }).unwrap()].protected);
        assert!(map.world.cells[map.world.index(CellPos { x: 22, y: 7 }).unwrap()].sealed);
        assert_eq!(map.reference_tick_range, (480, 750));
    }

    #[test]
    fn l01_authors_a_visible_terrace_ridge_cut_and_basin() {
        let map = load_campaign(MissionId::L01FirstFlow);
        let height = |pos| map.world.cells[map.world.index(pos).unwrap()].height_hu;
        assert!(height(CellPos { x: 4, y: 4 }) > height(CellPos { x: 14, y: 8 }));
        assert!(height(CellPos { x: 12, y: 7 }) > height(CellPos { x: 14, y: 8 }));
        assert!(height(CellPos { x: 14, y: 8 }) > height(CellPos { x: 22, y: 8 }));
        assert_eq!(height(CellPos { x: 22, y: 8 }), 0);
        assert_eq!(height(CellPos { x: 27, y: 16 }), 0);
    }

    #[test]
    fn l03_authored_start_contains_cistern_water_and_lava_source() {
        let map = load_campaign(MissionId::L03Firebreak);
        let water = map
            .world
            .cells
            .iter()
            .flat_map(|cell| cell.surface.iter())
            .any(|entry| entry.fluid == FluidId::Water && entry.volume_vu >= 750);
        assert!(water);
        assert_eq!(map.world.sources[0].rate_vu, 100);
    }

    #[test]
    fn authored_l02_and_l03_maps_do_not_autocomplete_without_player_builds() {
        for id in [MissionId::L02HoldingLine, MissionId::L03Firebreak] {
            let mut map = load_campaign(id);
            map.world.set_sources_enabled(true);
            let mut mission = crate::mission::MissionState::new(id);
            mission.start();
            for _ in 0..2_200 {
                apply_scheduled_events(&mut map.world, id);
                map.world.tick();
                mission.on_tick(&map.world);
                if mission.phase != crate::mission::MissionPhase::Active {
                    break;
                }
            }
            assert_ne!(
                mission.phase,
                crate::mission::MissionPhase::Success,
                "{id:?}: {mission:?}"
            );
        }
    }
    #[test]
    fn authored_source_surges_begin_and_end_on_schedule() {
        let mut map = load_campaign(MissionId::L02HoldingLine);
        assert_eq!(map.world.sources[0].rate_vu, 100);
        map.world.tick = 900;
        apply_scheduled_events(&mut map.world, MissionId::L02HoldingLine);
        assert_eq!(map.world.sources[0].rate_vu, 400);
        map.world.tick = 1_200;
        apply_scheduled_events(&mut map.world, MissionId::L02HoldingLine);
        assert_eq!(map.world.sources[0].rate_vu, 100);
    }
}
