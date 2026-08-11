use super::*;
fn config() -> GameConfig {
    GameConfig {
        game_name: "test".into(),
        display_name: "Test".into(),
        save_slot: "test".into(),
        version: "1".into(),
        world_width: 8,
        world_height: 8,
    }
}
#[test]
fn row_major_coordinates_are_stable() {
    let w = WorldState::new(8, 8);
    assert_eq!(w.index(CellPos { x: 2, y: 3 }), Some(26));
    assert!(w.index(CellPos { x: 8, y: 0 }).is_none());
}
#[test]
fn partitioning_does_not_change_tick_hash() {
    let c = config();
    let mut a = GameSession::new(&c);
    let mut b = GameSession::new(&c);
    a.time_control = TimeControl::OneX;
    b.time_control = TimeControl::OneX;
    for _ in 0..10 {
        a.update(0.1);
    }
    b.update(0.25);
    b.update(0.25);
    b.update(0.5);
    assert_eq!(a.state_hash(), b.state_hash());
}
#[test]
fn paused_session_does_not_tick() {
    let c = config();
    let mut s = GameSession::new(&c);
    s.update(10.0);
    assert_eq!(s.tick, 0);
}
#[test]
fn save_round_trip_is_exact() {
    let c = config();
    let mut s = GameSession::new(&c);
    s.tick();
    s.mission.on_tick(&s.simulation);
    s.mission.checkpoint();
    s.campaign.record_success(MissionId::L01FirstFlow, 700);
    let save = s.to_save("1");
    let restored = GameSession::from_save(save);
    assert_eq!(restored.state_hash(), s.state_hash());
    assert_eq!(restored.mission.checkpoint_tick, 1);
    assert_eq!(restored.campaign.best_ticks[0], Some(700));
}

#[test]
fn future_schema_is_rejected_with_recovery_message() {
    let c = config();
    let mut value = serde_json::to_value(GameSession::new(&c).to_save(&c.version)).unwrap();
    value["schema_version"] = Value::from(SAVE_SCHEMA_VERSION + 1);
    let error = migrate_save_value(value, &c).unwrap_err();
    assert!(error.contains("unsupported"));
}

#[test]
fn mismatched_content_version_is_rejected() {
    let c = config();
    let mut value = serde_json::to_value(GameSession::new(&c).to_save("0.9.0")).unwrap();
    value["version"] = Value::from("0.9.0");
    let error = migrate_save_value(value, &c).unwrap_err();
    assert!(error.contains("content version"));
}

#[test]
fn corrupted_save_is_rejected_without_constructing_a_session() {
    let error = migrate_save_value(Value::from("not a save"), &config()).unwrap_err();
    assert!(error.contains("malformed"));
}
