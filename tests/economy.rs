//! Finite fabrication, deposit recovery, power allocation, and refund contracts.

use planetfall_engineer::{
    devices::DeviceId,
    economy::{FabricationError, FabricationState, PowerLedger},
    simulation::{FluidId, SimulationWorld},
    state::CellPos,
};

#[test]
fn deposit_recovery_is_one_shot_and_balanced() {
    let mut world = SimulationWorld::new(4, 4);
    world.add_deposit("cache", CellPos { x: 1, y: 1 }, 28, "deposit_cache");
    assert_eq!(world.recover_deposit("cache"), Ok(28));
    let snapshot = serde_json::to_vec(&world).unwrap();
    assert_eq!(
        world.recover_deposit("cache"),
        Err(FabricationError::DepositDepleted("cache".into()))
    );
    assert_eq!(serde_json::to_vec(&world).unwrap(), snapshot);
    assert_eq!(world.fabrication.available_fu, 28);
    assert_eq!(world.fabrication.balance_error(), 0);
}

#[test]
fn reservations_cancel_and_failed_commit_preserves_stock() {
    let world = SimulationWorld::new(4, 4);
    let mut devices = planetfall_engineer::devices::DeviceSystem::default();
    let mut fabrication = FabricationState::new(20);
    devices
        .queue(
            &world,
            DeviceId::Pipe,
            CellPos { x: 1, y: 1 },
            0,
            &mut fabrication,
        )
        .unwrap();
    assert_eq!((fabrication.available_fu, fabrication.reserved_fu), (17, 3));
    devices
        .install_fixture(&world, DeviceId::Pipe, CellPos { x: 1, y: 1 }, 0)
        .unwrap();
    assert_eq!(
        devices.commit_plan(&world, &mut fabrication),
        Err(planetfall_engineer::devices::DeviceError::Occupied)
    );
    assert_eq!((fabrication.available_fu, fabrication.reserved_fu), (17, 3));
    assert!(devices.cancel_last_plan(&mut fabrication));
    assert_eq!((fabrication.available_fu, fabrication.reserved_fu), (20, 0));
    assert_eq!(fabrication.balance_error(), 0);
}

#[test]
fn pre_operation_and_operated_refunds_are_exact() {
    let world = SimulationWorld::new(4, 2);
    let mut devices = planetfall_engineer::devices::DeviceSystem::default();
    let mut fabrication = FabricationState::new(50);
    let unused = devices
        .place(
            &world,
            DeviceId::Pump,
            CellPos { x: 0, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    assert!(devices.remove(unused, &mut fabrication));
    assert_eq!(fabrication.available_fu, 50);

    let operated = devices
        .place(
            &world,
            DeviceId::Filter,
            CellPos { x: 2, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    let mut world = world;
    world.inject(CellPos { x: 2, y: 0 }, FluidId::ToxicSlurry, 500);
    devices.tick(&mut world);
    assert!(devices
        .devices
        .iter()
        .find(|device| device.entity_id == operated)
        .is_some_and(|device| device.operated_ticks == 1));
    assert!(devices.remove(operated, &mut fabrication));
    assert_eq!(fabrication.spent_fu, 26);
    assert_eq!(fabrication.refunded_fu, 22);
    assert_eq!(fabrication.available_fu, 46);
    assert_eq!(fabrication.balance_error(), 0);
}

#[test]
fn power_priority_is_whole_device_and_reports_transition() {
    let world = SimulationWorld::new(8, 2);
    let mut devices = planetfall_engineer::devices::DeviceSystem::default();
    let mut fabrication = FabricationState::new(100);
    let safety = devices
        .place(
            &world,
            DeviceId::Reservoir,
            CellPos { x: 0, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    let transport = devices
        .place(
            &world,
            DeviceId::Pump,
            CellPos { x: 3, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    let process = devices
        .place(
            &world,
            DeviceId::Filter,
            CellPos { x: 5, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    let mut ledger = PowerLedger::default();
    let brownouts = devices.allocate_power(&mut ledger, 4);
    assert_eq!(ledger.allocated_demand_eu, 4);
    assert_eq!(ledger.deficit_eu, 3);
    assert_eq!(brownouts, vec![(process, 3)]);
    assert!(
        devices
            .devices
            .iter()
            .find(|device| device.entity_id == safety)
            .unwrap()
            .powered
    );
    assert!(
        devices
            .devices
            .iter()
            .find(|device| device.entity_id == transport)
            .unwrap()
            .powered
    );
    assert!(
        !devices
            .devices
            .iter()
            .find(|device| device.entity_id == process)
            .unwrap()
            .powered
    );
}

#[test]
fn turbine_output_is_available_only_on_the_following_tick() {
    let mut world = SimulationWorld::new(4, 2);
    let mut devices = std::mem::take(&mut world.devices);
    let mut fabrication = FabricationState::new(20);
    devices
        .place(
            &world,
            DeviceId::FlowTurbine,
            CellPos { x: 1, y: 0 },
            0,
            &mut fabrication,
        )
        .unwrap();
    world.devices = devices;
    world.inject(CellPos { x: 1, y: 0 }, FluidId::Steam, 500);
    world.tick();
    assert_eq!(world.power.turbine_generation_eu, 0);
    let measured_output = world.devices.devices[0].power_generated;
    assert!(measured_output > 0);
    world.tick();
    assert_eq!(world.power.turbine_generation_eu, measured_output);
}
