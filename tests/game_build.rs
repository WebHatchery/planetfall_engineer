//! Logical hit regions for the visible build and time controls.

use planetfall_engineer::devices::DeviceId;
use planetfall_engineer::game_build::{
    build_click_action, build_click_at, field_control_action, field_control_at, BuildClick,
    FieldControl,
};
use planetfall_engineer::state::TimeControl;
use planetfall_engineer::ui_action::{pause_action_at, UiAction};

#[test]
fn every_visible_build_control_maps_to_its_action() {
    assert_eq!(
        build_click_at(1040.0, 354.0),
        Some(BuildClick::Palette(DeviceId::Channel))
    );
    assert_eq!(
        build_click_at(1200.0, 450.0),
        Some(BuildClick::Palette(DeviceId::RuneRelay))
    );
    assert_eq!(build_click_at(1040.0, 480.0), Some(BuildClick::Rotate));
    assert_eq!(build_click_at(1120.0, 480.0), Some(BuildClick::Commit));
    assert_eq!(build_click_at(1200.0, 480.0), Some(BuildClick::Cancel));
}

#[test]
fn every_time_button_maps_to_the_expected_speed() {
    assert_eq!(
        build_click_at(1040.0, 507.0),
        Some(BuildClick::Time(TimeControl::Paused))
    );
    assert_eq!(
        build_click_at(1100.0, 507.0),
        Some(BuildClick::Time(TimeControl::OneX))
    );
    assert_eq!(
        build_click_at(1160.0, 507.0),
        Some(BuildClick::Time(TimeControl::TwoX))
    );
    assert_eq!(
        build_click_at(1220.0, 507.0),
        Some(BuildClick::Time(TimeControl::FourX))
    );
    assert_eq!(build_click_at(900.0, 507.0), None);
}

#[test]
fn every_field_action_has_a_visible_touch_target() {
    assert_eq!(field_control_at(1040.0, 546.0), Some(FieldControl::Inspect));
    assert_eq!(
        field_control_at(1200.0, 546.0),
        Some(FieldControl::Excavate)
    );
    assert_eq!(field_control_at(1040.0, 580.0), Some(FieldControl::Raise));
    assert_eq!(field_control_at(1200.0, 580.0), Some(FieldControl::Seal));
    assert_eq!(field_control_at(1040.0, 614.0), Some(FieldControl::Gate(0)));
    assert_eq!(
        field_control_at(1100.0, 614.0),
        Some(FieldControl::Gate(2_500))
    );
    assert_eq!(
        field_control_at(1160.0, 614.0),
        Some(FieldControl::Gate(5_000))
    );
    assert_eq!(
        field_control_at(1220.0, 614.0),
        Some(FieldControl::Gate(10_000))
    );
    assert_eq!(field_control_at(1040.0, 648.0), Some(FieldControl::Recover));
    assert_eq!(
        field_control_at(1200.0, 648.0),
        Some(FieldControl::ToggleDevice)
    );
}

#[test]
fn visible_controls_emit_central_actions() {
    assert_eq!(build_click_action(BuildClick::Commit), UiAction::CommitPlan);
    assert_eq!(
        field_control_action(FieldControl::Recover),
        UiAction::RecoverDeposit
    );
    assert_eq!(
        field_control_action(FieldControl::ToggleDevice),
        UiAction::ToggleSelectedDevice
    );
}

#[test]
fn pause_overlay_actions_have_logical_touch_targets() {
    for (x, expected) in [
        (480.0, UiAction::SetTime(TimeControl::OneX)),
        (600.0, UiAction::SetTime(TimeControl::TwoX)),
        (720.0, UiAction::SetTime(TimeControl::FourX)),
        (480.0, UiAction::Save),
        (610.0, UiAction::Load),
        (740.0, UiAction::ResetMission),
    ] {
        let y = if expected == UiAction::Save
            || expected == UiAction::Load
            || expected == UiAction::ResetMission
        {
            306.0
        } else {
            258.0
        };
        assert_eq!(pause_action_at(x, y), Some(expected));
    }
}
