use super::*;
use crate::simulation::{FluidId, SimulationWorld};
#[test]
fn clean_campaign_unlocks_only_l01() {
    let progress = CampaignProgress::default();
    assert_eq!(progress.unlocked, [true, false, false]);
}
#[test]
fn tutorial_rejects_locked_state_changes_and_advances_on_events() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    assert_eq!(
        mission.admit(CommandKind::QueueDevice(DeviceId::Channel)),
        Admission::TutorialLocked
    );
    assert_eq!(
        mission.admit(CommandKind::DismissPrompt),
        Admission::Accepted
    );
    assert_eq!(
        mission.tutorial.as_ref().unwrap().current_step_id,
        "tutorial_l01_move_camera"
    );
}
#[test]
fn skip_completes_tutorial_without_mutating_budget() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    let budget = mission.budget;
    assert_eq!(mission.skip_tutorial(), Admission::Accepted);
    assert!(mission.tutorial.as_ref().unwrap().is_complete());
    assert_eq!(mission.budget, budget);
}
#[test]
fn normal_tutorial_path_reaches_completion() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    for (index, command) in [
        CommandKind::DismissPrompt,
        CommandKind::Camera,
        CommandKind::Select,
        CommandKind::Inspect,
        CommandKind::DismissPrompt,
        CommandKind::SetPaused,
        CommandKind::SelectTerrain,
        CommandKind::Select,
        CommandKind::SelectTerrain,
        CommandKind::Select,
        CommandKind::SelectTerrain,
        CommandKind::SetTimeRunning,
        CommandKind::SetTimeRunning,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            mission.admit(command),
            Admission::Accepted,
            "step {index} current {:?}",
            mission.tutorial.as_ref().unwrap().current_step_id
        );
    }
    assert!(mission.tutorial.as_ref().unwrap().is_complete());
}

#[test]
fn inspection_explanation_requires_inspection_before_dismissal() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    for command in [
        CommandKind::DismissPrompt,
        CommandKind::Camera,
        CommandKind::Select,
    ] {
        assert_eq!(mission.admit(command), Admission::Accepted);
    }
    assert_eq!(
        mission.admit(CommandKind::DismissPrompt),
        Admission::Accepted
    );
    assert_eq!(
        mission.tutorial.as_ref().unwrap().current_step_id,
        "tutorial_l01_inspect_grade"
    );
    assert_eq!(mission.admit(CommandKind::Inspect), Admission::Accepted);
    assert_eq!(
        mission.admit(CommandKind::DismissPrompt),
        Admission::Accepted
    );
    assert_eq!(
        mission.tutorial.as_ref().unwrap().current_step_id,
        "tutorial_l01_pause_plan"
    );
}
#[test]
fn success_requires_stability_and_terminal_state_stops_ticks() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    let mut world = SimulationWorld::new(32, 20);
    world.inject(crate::state::CellPos { x: 28, y: 9 }, FluidId::Water, 8_000);
    for _ in 0..100 {
        world.tick += 1;
        mission.on_tick(&world);
    }
    assert_eq!(mission.phase, MissionPhase::Success);
    let tick = mission.tick;
    mission.on_tick(&world);
    assert_eq!(mission.tick, tick);
}
#[test]
fn firebreak_progress_uses_formed_rock() {
    let mut mission = MissionState::new(MissionId::L03Firebreak);
    mission.start();
    let mut world = SimulationWorld::new(48, 30);
    let shelf = crate::state::CellPos { x: 22, y: 14 };
    world.inject(shelf, FluidId::Water, 4_000);
    world.inject(shelf, FluidId::Lava, 4_000);
    world.tick();
    mission.on_tick(&world);
    assert!(mission.objective_progress >= 250);
    assert_eq!(mission.stability_ticks, 0);
}
#[test]
fn campaign_progress_ignores_material_outside_its_authored_zone() {
    let mut l01 = MissionState::new(MissionId::L01FirstFlow);
    l01.start();
    let mut basin = SimulationWorld::new(32, 20);
    basin.inject(crate::state::CellPos { x: 2, y: 2 }, FluidId::Water, 8_000);
    l01.on_tick(&basin);
    assert_eq!(l01.objective_progress, 0);

    let mut l03 = MissionState::new(MissionId::L03Firebreak);
    l03.start();
    let mut caldera = SimulationWorld::new(48, 30);
    let outside = caldera.index(crate::state::CellPos { x: 2, y: 2 }).unwrap();
    let cell = &mut caldera.cells[outside];
    cell.formed_rock_vu = 8_000;
    l03.on_tick(&caldera);
    assert_eq!(l03.objective_progress, 0);
}
#[test]
fn authored_hazard_fails_before_success() {
    let mut mission = MissionState::new(MissionId::L01FirstFlow);
    mission.start();
    let mut world = SimulationWorld::new(32, 20);
    let beacon = crate::state::CellPos { x: 24, y: 8 };
    let index = world.index(beacon).unwrap();
    world.cells[index].sealed = true;
    world.inject(beacon, FluidId::Water, 1_500);
    for _ in 0..10 {
        mission.on_tick(&world);
    }
    assert_eq!(mission.phase, MissionPhase::Failure);
    assert_eq!(
        mission.failure_reason.as_deref(),
        Some("protected beacon flooded")
    );
}
#[test]
fn authored_alert_levels_escalate_before_failure() {
    let mut mission = MissionState::new(MissionId::L02HoldingLine);
    mission.start();
    let mut world = SimulationWorld::new(40, 24);
    world.tick = 600;
    mission.on_tick(&world);
    assert_eq!(mission.alert_level, AlertLevel::Advisory);
    world.tick = 800;
    mission.on_tick(&world);
    assert_eq!(mission.alert_level, AlertLevel::Warning);
    let camp = crate::state::CellPos { x: 26, y: 16 };
    world.inject(camp, FluidId::Water, 1_500);
    mission.on_tick(&world);
    assert_eq!(mission.alert_level, AlertLevel::Critical);
}
#[test]
fn completion_unlocks_next_campaign_level_and_keeps_best_time() {
    let mut progress = CampaignProgress::default();
    progress.record_success(MissionId::L01FirstFlow, 700);
    progress.record_success(MissionId::L01FirstFlow, 800);
    assert_eq!(progress.unlocked, [true, true, false]);
    assert_eq!(progress.best_ticks[0], Some(700));
}
