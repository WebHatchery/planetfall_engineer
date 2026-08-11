use super::*;
fn pos(x: u16, y: u16) -> CellPos {
    CellPos { x, y }
}
#[test]
fn terrain_actions_obey_protection_and_capacity() {
    let mut world = SimulationWorld::new(3, 1);
    world.definitions[0].protected = true;
    assert_eq!(
        world.terrain_edit(pos(0, 0), TerrainAction::Excavate),
        Err(TerrainError::Protected)
    );
    world.inject(pos(1, 0), FluidId::Water, 100);
    assert_eq!(
        world.terrain_edit(pos(1, 0), TerrainAction::Raise),
        Err(TerrainError::Capacity)
    );
    assert!(world
        .terrain_edit(pos(1, 0), TerrainAction::Excavate)
        .is_ok());
}
#[test]
fn water_flows_downhill_and_respects_limit() {
    let mut world = SimulationWorld::new(2, 1);
    world.cells[0].height_hu = 2_000;
    world.cells[1].height_hu = 0;
    world.inject(pos(0, 0), FluidId::Water, 1_000);
    world.tick();
    assert_eq!(volume(&world.cells[0].surface, FluidId::Water), 600);
    assert_eq!(volume(&world.cells[1].surface, FluidId::Water), 400);
}

#[test]
fn shallow_surface_gradient_moves_only_the_hydraulic_proposal() {
    let mut world = SimulationWorld::new(2, 1);
    world.cells[0].height_hu = 0;
    world.cells[1].height_hu = 0;
    world.inject(pos(0, 0), FluidId::Water, 20);
    world.inject(pos(1, 0), FluidId::Water, 4);
    world.flow_surface();
    assert_eq!(volume(&world.cells[0].surface, FluidId::Water), 16);
    assert_eq!(volume(&world.cells[1].surface, FluidId::Water), 8);
}

#[test]
fn sealed_floor_prevents_contamination_not_surface_flow() {
    let mut world = SimulationWorld::new(2, 1);
    world.cells[1].sealed = true;
    world.inject(pos(0, 0), FluidId::Water, 1_000);
    world.flow_surface();
    assert_eq!(volume(&world.cells[1].surface, FluidId::Water), 250);
}

#[test]
fn steam_competing_transfers_share_one_material_cap() {
    let mut world = SimulationWorld::new(3, 3);
    for definition in &mut world.definitions {
        definition.ambient_temperature_dk = WATER_BOIL_DK + 100;
    }
    world.inject(pos(1, 1), FluidId::Steam, 2_000);
    world.flow_steam();
    let center = volume(&world.cells[4].airborne, FluidId::Steam);
    let neighbors: Vec<u32> = [pos(1, 0), pos(2, 1), pos(1, 2), pos(0, 1)]
        .into_iter()
        .map(|position| {
            volume(
                &world.cells[world.index(position).unwrap()].airborne,
                FluidId::Steam,
            )
        })
        .collect();
    assert_eq!(center, 1_700);
    assert_eq!(neighbors.iter().sum::<u32>(), 300);
    assert!(neighbors.iter().all(|amount| (74..=76).contains(amount)));
}

#[test]
fn condensation_preserves_steam_blocked_by_surface_capacity() {
    let mut world = SimulationWorld::new(1, 1);
    world.inject(pos(0, 0), FluidId::Water, CELL_CAPACITY_VU - 50);
    world.inject(pos(0, 0), FluidId::Steam, 200);
    world.flow_steam();
    assert_eq!(world.total_material_volume(), CELL_CAPACITY_VU as u64 + 150);
    assert_eq!(volume(&world.cells[0].airborne, FluidId::Steam), 150);
}
#[test]
fn mixed_surface_transfer_preserves_material_proportions() {
    let mut world = SimulationWorld::new(2, 1);
    world.cells[0].height_hu = 2_000;
    world.cells[1].height_hu = 0;
    world.inject(pos(0, 0), FluidId::Water, 600);
    world.inject(pos(0, 0), FluidId::Lava, 400);
    world.flow_surface();
    assert_eq!(volume(&world.cells[1].surface, FluidId::Water), 72);
    assert_eq!(volume(&world.cells[1].surface, FluidId::Lava), 48);
}
#[test]
fn lava_and_water_make_steam_and_rock() {
    let mut world = SimulationWorld::new(1, 1);
    world.inject(pos(0, 0), FluidId::Water, 500);
    world.inject(pos(0, 0), FluidId::Lava, 500);
    world.tick();
    assert_eq!(volume(&world.cells[0].surface, FluidId::Water), 250);
    assert_eq!(volume(&world.cells[0].surface, FluidId::Lava), 250);
    assert_eq!(volume(&world.cells[0].airborne, FluidId::Steam), 250);
    assert_eq!(world.cells[0].pending_rock_vu, 250);
    assert!(world.events.iter().any(|event| matches!(event, SimEvent::MaterialReacted { reaction, .. } if reaction == "water_lava")));
}
#[test]
fn slurry_contaminates_unsealed_ground_and_lava_vitrifies() {
    let mut world = SimulationWorld::new(1, 1);
    world.inject(pos(0, 0), FluidId::ToxicSlurry, 500);
    world.inject(pos(0, 0), FluidId::Lava, 500);
    world.tick();
    assert_eq!(world.cells[0].ground_contamination_bp, 10_000);
    assert_eq!(world.cells[0].pending_vitrified_vu, 240);
}
#[test]
fn source_backpressure_is_explicit() {
    let mut world = SimulationWorld::new(1, 1);
    world.inject(pos(0, 0), FluidId::Water, 9_000);
    assert!(world.events.iter().any(|event| matches!(
        event,
        SimEvent::SourceBackpressure {
            accepted_vu: 8_000,
            ..
        }
    )));
}
#[test]
fn authored_source_stays_stopped_until_enabled() {
    let mut world = SimulationWorld::new(2, 1);
    world.add_source(pos(0, 0), FluidId::Water, 180);
    world.tick();
    assert_eq!(
        world
            .cells
            .iter()
            .map(|cell| volume(&cell.surface, FluidId::Water))
            .sum::<u32>(),
        0
    );
    world.set_sources_enabled(true);
    world.tick();
    assert_eq!(
        world
            .cells
            .iter()
            .map(|cell| volume(&cell.surface, FluidId::Water))
            .sum::<u32>(),
        180
    );
}
#[test]
fn surface_outlet_drains_material_and_balances_the_ledger() {
    let mut world = SimulationWorld::new(1, 1);
    world.definitions[0].surface_drain_rate_vu = 75;
    world.inject(pos(0, 0), FluidId::Water, 100);
    world.tick();
    assert_eq!(world.cells[0].surface_volume(), 25);
    assert_eq!(world.ledger.drained, 75);
    assert_eq!(world.mass_balance_error(), 0);
}
#[test]
fn material_balance_includes_reaction_products() {
    let mut world = SimulationWorld::new(1, 1);
    world.inject(pos(0, 0), FluidId::Water, 500);
    world.inject(pos(0, 0), FluidId::Lava, 500);
    world.tick();
    assert_eq!(world.mass_balance_error(), 0);
}
#[test]
fn gas_blocked_cells_reject_steam_transfers() {
    let mut world = SimulationWorld::new(2, 1);
    world.definitions[1].gas_blocked = true;
    world.definitions[0].ambient_temperature_dk = 4_730;
    world.definitions[1].ambient_temperature_dk = 4_730;
    world.inject(pos(0, 0), FluidId::Steam, 1_000);
    world.tick();
    assert_eq!(volume(&world.cells[1].airborne, FluidId::Steam), 0);
    assert!(volume(&world.cells[0].airborne, FluidId::Steam) > 0);
}
