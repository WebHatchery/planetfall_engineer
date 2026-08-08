//! Authored campaign map constructors and reference fixtures.

use crate::{
    mission::MissionId,
    simulation::{CellDefinition, FluidId, SimulationWorld},
    state::CellPos,
};

#[derive(Debug, Clone)]
pub struct CampaignMap {
    pub id: MissionId,
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
        id,
        world,
        budget,
        reference_tick_range,
    }
}

fn author_l01(world: &mut SimulationWorld) {
    set_ambient(world, 3_030);
    world.add_source(CellPos { x: 4, y: 4 }, FluidId::Water, 180);
    protect(world, CellPos { x: 10, y: 6 });
    for x in 12..=18 {
        set_height(world, CellPos { x, y: 7 }, 1_500);
    }
    for x in 14..=18 {
        set_height(world, CellPos { x, y: 8 }, 1_250);
    }
    for y in 7..=10 {
        for x in 22..=25 {
            set_sealed(world, CellPos { x, y });
        }
    }
}

fn author_l02(world: &mut SimulationWorld) {
    set_ambient(world, 2_930);
    world.add_source(CellPos { x: 5, y: 16 }, FluidId::Water, 100);
    for y in 14..=18 {
        for x in 3..=7 {
            set_height(world, CellPos { x, y }, 0);
        }
    }
    for y in 14..=18 {
        for x in 24..=28 {
            protect(world, CellPos { x, y });
        }
    }
    for y in 7..=9 {
        for x in 31..=36 {
            set_sealed(world, CellPos { x, y });
        }
    }
}

fn author_l03(world: &mut SimulationWorld) {
    set_ambient(world, 3_230);
    world.add_source(CellPos { x: 5, y: 15 }, FluidId::Lava, 100);
    // The cistern's first controlled pocket makes the documented water/lava
    // reaction observable from the authored campaign start.
    world.inject(CellPos { x: 22, y: 14 }, FluidId::Water, 3_000);
    world.inject(CellPos { x: 23, y: 14 }, FluidId::Lava, 3_000);
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
}

pub fn seed_reference_materials(map: &mut CampaignMap) {
    match map.id {
        MissionId::L01FirstFlow => map
            .world
            .inject(CellPos { x: 4, y: 4 }, FluidId::Water, 180),
        MissionId::L02HoldingLine => {
            map.world
                .inject(CellPos { x: 5, y: 16 }, FluidId::Water, 8_000)
        }
        MissionId::L03Firebreak => {
            map.world
                .inject(CellPos { x: 5, y: 15 }, FluidId::Lava, 100);
            map.world
                .inject(CellPos { x: 9, y: 4 }, FluidId::Water, 12_000);
        }
    }
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
    fn l03_authored_start_contains_water_and_lava_reaction_materials() {
        let map = load_campaign(MissionId::L03Firebreak);
        let water = map
            .world
            .cells
            .iter()
            .flat_map(|cell| cell.surface.iter())
            .any(|entry| entry.fluid == FluidId::Water && entry.volume_vu >= 3_000);
        let lava = map
            .world
            .cells
            .iter()
            .flat_map(|cell| cell.surface.iter())
            .any(|entry| entry.fluid == FluidId::Lava && entry.volume_vu >= 3_000);
        assert!(water && lava);
        assert_eq!(map.world.sources[0].rate_vu, 100);
    }
    #[test]
    fn reference_material_seed_is_deterministic() {
        let mut first = load_campaign(MissionId::L03Firebreak);
        let mut second = load_campaign(MissionId::L03Firebreak);
        seed_reference_materials(&mut first);
        seed_reference_materials(&mut second);
        assert_eq!(
            serde_json::to_vec(&first.world).unwrap(),
            serde_json::to_vec(&second.world).unwrap()
        );
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
