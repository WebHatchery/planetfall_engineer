use super::*;
#[test]
fn lab_has_four_active_and_five_reserved_bays() {
    assert_eq!(LAB_BAYS.iter().filter(|bay| bay.fluid.is_some()).count(), 4);
    assert_eq!(LAB_BAYS.iter().filter(|bay| bay.fluid.is_none()).count(), 5);
}
#[test]
fn lab_automatic_scenario_passes() {
    let report = FluidsLab::new().automatic_scenario();
    assert!(report.passed, "{report:?}");
    assert!(report.reaction_events > 0);
}
#[test]
fn lab_replay_hash_is_stable() {
    let first = FluidsLab::new().automatic_scenario();
    let second = FluidsLab::new().automatic_scenario();
    assert_eq!(first, second);
}
#[test]
fn reset_restores_initial_hash() {
    let mut lab = FluidsLab::new();
    let initial = state_hash(&lab.world);
    lab.world.tick();
    lab.reset();
    assert_eq!(state_hash(&lab.world), initial);
}
#[test]
fn synthetic_worst_case_soak_is_deterministic_and_conservative() {
    fn make_world() -> SimulationWorld {
        let mut world = SimulationWorld::new(64, 48);
        for y in 0..48 {
            for x in 0..64 {
                let pos = CellPos { x, y };
                world.inject(pos, FluidId::Water, 500);
                world.inject(pos, FluidId::Lava, 500);
                world.inject(pos, FluidId::ToxicSlurry, 500);
                world.inject(pos, FluidId::Steam, 500);
            }
        }
        world
    }
    let mut first = make_world();
    let mut second = make_world();
    for _ in 0..10_000 {
        first.tick();
        second.tick();
    }
    assert_eq!(first.mass_balance_error(), 0);
    assert_eq!(second.mass_balance_error(), 0);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
}
