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
    assert!(map.world.definitions[map.world.index(CellPos { x: 24, y: 8 }).unwrap()].protected);
    assert!(map.world.cells[map.world.index(CellPos { x: 26, y: 7 }).unwrap()].sealed);
    assert!(map.world.cells[map.world.index(CellPos { x: 26, y: 7 }).unwrap()].contained);
    assert_eq!(map.reference_tick_range, (900, 1_400));
    assert_eq!(map.world.sources.len(), 1);
    assert_eq!(map.world.sources[0].position, CellPos { x: 2, y: 9 });
    assert_eq!(map.world.sources[0].rate_vu, 20);
    assert_eq!(
        map.world.definitions[map.world.index(CellPos { x: 8, y: 14 }).unwrap()]
            .surface_drain_rate_vu,
        800
    );
}

#[test]
fn l01_authors_three_runoff_cuts_and_a_far_side_dam() {
    let map = load_campaign(MissionId::L01FirstFlow);
    let height = |pos| map.world.cells[map.world.index(pos).unwrap()].height_hu;
    assert!(height(CellPos { x: 2, y: 9 }) > height(CellPos { x: 25, y: 9 }));
    for x in [8, 14, 20] {
        assert!(height(CellPos { x, y: 10 }) < height(CellPos { x, y: 9 }));
    }
    assert_eq!(height(CellPos { x: 28, y: 9 }), 600);
}

#[test]
fn l01_barriers_are_required_to_feed_the_far_side_dam() {
    let mut map = load_campaign(MissionId::L01FirstFlow);
    map.world.set_sources_enabled(true);
    for _ in 0..1_200 {
        map.world.tick();
    }
    let untouched_water: u32 = (26..=30)
        .flat_map(|x| (7..=11).map(move |y| CellPos { x, y }))
        .map(|pos| map.world.cells[map.world.index(pos).unwrap()].surface_volume())
        .sum();
    assert_eq!(
        untouched_water, 0,
        "untouched runoff must never reach the dam"
    );

    let mut solved = load_campaign(MissionId::L01FirstFlow);
    for pos in [
        CellPos { x: 8, y: 10 },
        CellPos { x: 14, y: 10 },
        CellPos { x: 20, y: 10 },
    ] {
        solved
            .world
            .terrain_edit(pos, crate::simulation::TerrainAction::Raise)
            .unwrap();
    }
    solved.world.set_sources_enabled(true);
    for _ in 0..1_200 {
        solved.world.tick();
    }
    let dam_water: u32 = (26..=30)
        .flat_map(|x| (7..=11).map(move |y| CellPos { x, y }))
        .map(|pos| solved.world.cells[solved.world.index(pos).unwrap()].surface_volume())
        .sum();
    assert!(
        dam_water >= 6_000,
        "three barriers must fill the far-side dam: {dam_water} vU"
    );
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
    assert_eq!(map.world.sources.len(), 1);
    assert_eq!(map.world.sources[0].position, CellPos { x: 5, y: 15 });
    assert_eq!(map.world.sources[0].fluid, FluidId::Lava);
    assert_eq!(map.world.sources[0].rate_vu, 100);
}

#[test]
fn l02_has_no_hidden_objective_source() {
    let map = load_campaign(MissionId::L02HoldingLine);
    assert_eq!(map.world.sources.len(), 1);
    assert_eq!(map.world.sources[0].position, CellPos { x: 5, y: 16 });
    assert_eq!(map.world.sources[0].rate_vu, 100);
    assert!((31..=36).all(|x| {
        (7..=9).all(|y| map.world.cells[map.world.index(CellPos { x, y }).unwrap()].contained)
    }));
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
