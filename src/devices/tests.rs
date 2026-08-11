use super::showcase::run_showcase;
use super::*;
#[test]
fn all_ten_devices_have_unique_showcases() {
    let reports: Vec<_> = DeviceId::ALL.into_iter().map(run_showcase).collect();
    assert_eq!(reports.len(), 10);
    assert!(reports.iter().all(|report| report.placed));
    assert_eq!(
        reports
            .iter()
            .map(|report| report.device)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        10
    );
}
#[test]
fn showcase_index_has_exactly_one_named_map_per_device() {
    assert_eq!(SHOWCASE_MAPS.len(), DeviceId::ALL.len());
    assert_eq!(
        SHOWCASE_MAPS
            .iter()
            .map(|showcase| showcase.device)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        10
    );
    assert!(SHOWCASE_MAPS
        .iter()
        .all(|showcase| showcase.map_id == format!("device_{}", showcase.device.name())));
}
#[test]
fn footprint_rotation_and_overlap_are_rejected() {
    let world = SimulationWorld::new(4, 4);
    let mut devices = DeviceSystem::default();
    assert!(devices
        .place(&world, DeviceId::Reservoir, CellPos { x: 1, y: 1 }, 3, 100)
        .is_ok());
    assert_eq!(
        devices.place(&world, DeviceId::Channel, CellPos { x: 1, y: 1 }, 0, 100),
        Err(DeviceError::Occupied)
    );
    assert_eq!(
        devices.place(&world, DeviceId::Channel, CellPos { x: 3, y: 3 }, 4, 100),
        Err(DeviceError::InvalidRotation)
    );
}
#[test]
fn budget_and_protected_placement_are_explained() {
    let mut world = SimulationWorld::new(4, 4);
    world.definitions[0].protected = true;
    let mut devices = DeviceSystem::default();
    assert_eq!(
        devices.place(&world, DeviceId::Pump, CellPos { x: 0, y: 0 }, 0, 100),
        Err(DeviceError::Protected)
    );
    assert_eq!(
        devices.place(&world, DeviceId::Pump, CellPos { x: 1, y: 1 }, 0, 10),
        Err(DeviceError::InsufficientBudget)
    );
}
#[test]
fn turbine_and_filter_have_renderer_independent_state() {
    let turbine = run_showcase(DeviceId::FlowTurbine);
    let filter = run_showcase(DeviceId::Filter);
    assert!(turbine.active_after_tick);
    assert!(filter.active_after_tick);
    assert_ne!(turbine.state_hash, filter.state_hash);
}
#[test]
fn pump_moves_water_in_rotated_direction() {
    let mut world = SimulationWorld::new(3, 1);
    let anchor = CellPos { x: 0, y: 0 };
    let mut placed = std::mem::take(&mut world.devices);
    placed
        .place(&world, DeviceId::Pump, anchor, 0, 100)
        .unwrap();
    world.devices = placed;
    world.inject(anchor, FluidId::Water, 500);
    let mut devices = std::mem::take(&mut world.devices);
    devices.tick(&mut world);
    assert_eq!(world.cells[0].surface_volume(), 250);
    assert_eq!(world.cells[1].surface_volume(), 250);
    assert!(devices.devices[0].active);
    world.devices = devices;
}
#[test]
fn filter_removes_contamination_without_losing_volume() {
    let mut world = SimulationWorld::new(2, 1);
    let mut devices = std::mem::take(&mut world.devices);
    devices
        .place(&world, DeviceId::Filter, CellPos { x: 0, y: 0 }, 0, 100)
        .unwrap();
    world.devices = devices;
    world.inject(CellPos { x: 0, y: 0 }, FluidId::ToxicSlurry, 500);
    let mut devices = std::mem::take(&mut world.devices);
    devices.tick(&mut world);
    let entry = &world.cells[0].surface[0];
    assert_eq!(entry.volume_vu, 500);
    assert_eq!(entry.contamination_bp, 7_500);
    assert!(devices.devices[0].active);
    world.devices = devices;
}
#[test]
fn queued_plans_reserve_budget_and_commit_atomically() {
    let mut world = SimulationWorld::new(4, 2);
    let mut devices = std::mem::take(&mut world.devices);
    assert_eq!(
        devices.queue(&world, DeviceId::Channel, CellPos { x: 0, y: 0 }, 0, 5),
        Ok(0)
    );
    assert_eq!(
        devices.queue(&world, DeviceId::Pipe, CellPos { x: 1, y: 0 }, 0, 5),
        Ok(1)
    );
    assert_eq!(devices.reserved_budget, 5);
    let committed = devices.commit_plan(&world, 5).unwrap();
    assert_eq!(committed, vec![0, 1]);
    assert!(devices.queued.is_empty());
    assert_eq!(devices.budget_spent, 5);
    world.devices = devices;
}
#[test]
fn cancelling_a_plan_releases_reserved_budget() {
    let world = SimulationWorld::new(2, 1);
    let mut devices = DeviceSystem::default();
    devices
        .queue(&world, DeviceId::Channel, CellPos { x: 0, y: 0 }, 0, 10)
        .unwrap();
    assert!(devices.cancel_last_plan());
    assert_eq!(devices.reserved_budget, 0);
    assert!(!devices.cancel_last_plan());
}
#[test]
fn floodgate_setting_changes_edge_flow_factor() {
    let world = SimulationWorld::new(2, 1);
    let mut devices = DeviceSystem::default();
    devices
        .place(&world, DeviceId::Floodgate, CellPos { x: 0, y: 0 }, 0, 100)
        .unwrap();
    assert_eq!(
        devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }),
        10_000
    );
    devices.set_selected_gate(CellPos { x: 0, y: 0 }, 5_000);
    assert_eq!(
        devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }),
        5_000
    );
    devices.set_selected_gate(CellPos { x: 0, y: 0 }, 0);
    assert_eq!(
        devices.surface_flow_factor(CellPos { x: 0, y: 0 }, CellPos { x: 1, y: 0 }),
        0
    );
}
#[test]
fn closed_gate_stops_surface_transfer_until_reopened() {
    let mut world = SimulationWorld::new(2, 1);
    world.cells[0].height_hu = 2_000;
    world.cells[1].height_hu = 0;
    let mut devices = std::mem::take(&mut world.devices);
    devices
        .place(&world, DeviceId::Floodgate, CellPos { x: 0, y: 0 }, 0, 100)
        .unwrap();
    devices.set_selected_gate(CellPos { x: 0, y: 0 }, 0);
    world.devices = devices;
    world.inject(CellPos { x: 0, y: 0 }, FluidId::Water, 1_000);
    world.tick();
    assert_eq!(world.cells[1].surface_volume(), 0);
    let mut devices = std::mem::take(&mut world.devices);
    devices.set_selected_gate(CellPos { x: 0, y: 0 }, 10_000);
    world.devices = devices;
    world.inject(CellPos { x: 0, y: 0 }, FluidId::Water, 1_000);
    world.tick();
    assert!(world.cells[1].surface_volume() > 0);
}

#[test]
fn pump_transports_into_a_connected_pipe_endpoint() {
    let mut world = SimulationWorld::new(5, 2);
    let mut devices = std::mem::take(&mut world.devices);
    devices
        .place(&world, DeviceId::Pump, CellPos { x: 0, y: 0 }, 0, 100)
        .unwrap();
    devices
        .place(&world, DeviceId::Pipe, CellPos { x: 1, y: 0 }, 0, 100)
        .unwrap();
    devices
        .place(&world, DeviceId::Pipe, CellPos { x: 2, y: 0 }, 0, 100)
        .unwrap();
    devices
        .place(&world, DeviceId::Reservoir, CellPos { x: 3, y: 0 }, 0, 100)
        .unwrap();
    assert_eq!(
        pipe_endpoint(&devices.devices, CellPos { x: 1, y: 0 }, (1, 0), None,),
        Some(CellPos { x: 3, y: 0 })
    );
    world.inject(CellPos { x: 0, y: 0 }, FluidId::Water, 500);
    devices.tick(&mut world);
    assert_eq!(devices.devices[3].stored_vu, 250);
    assert_eq!(world.cells[0].surface_volume(), 250);
}

#[test]
fn reservoir_releases_beyond_a_connected_pipe_run() {
    let mut world = SimulationWorld::new(8, 3);
    let mut devices = DeviceSystem::default();
    devices
        .place(&world, DeviceId::Reservoir, CellPos { x: 1, y: 1 }, 0, 100)
        .unwrap();
    for x in 3..=5 {
        devices
            .place(&world, DeviceId::Pipe, CellPos { x, y: 1 }, 0, 100)
            .unwrap();
    }
    let reservoir = devices
        .devices
        .iter_mut()
        .find(|device| device.device == DeviceId::Reservoir)
        .unwrap();
    reservoir.stored_vu = 1_000;
    reservoir.stored_fluid = Some(FluidId::Water);
    reservoir.setting_bp = 5_000;
    devices.tick(&mut world);
    let destination = world.index(CellPos { x: 6, y: 1 }).unwrap();
    assert_eq!(world.cells[destination].surface_volume(), 400);
}

#[test]
fn relay_requires_an_operating_turbine_and_pipe_link() {
    let mut world = SimulationWorld::new(5, 2);
    let mut devices = std::mem::take(&mut world.devices);
    devices
        .place(
            &world,
            DeviceId::FlowTurbine,
            CellPos { x: 0, y: 0 },
            0,
            100,
        )
        .unwrap();
    devices
        .place(&world, DeviceId::Pipe, CellPos { x: 1, y: 0 }, 0, 100)
        .unwrap();
    devices
        .place(&world, DeviceId::RuneRelay, CellPos { x: 2, y: 0 }, 0, 100)
        .unwrap();
    world.inject(CellPos { x: 0, y: 0 }, FluidId::Steam, 500);
    world.inject(CellPos { x: 2, y: 0 }, FluidId::Water, 100);
    devices.tick(&mut world);
    let relay = devices
        .devices
        .iter()
        .find(|device| device.device == DeviceId::RuneRelay)
        .unwrap();
    assert!(relay.powered && relay.active);
}
