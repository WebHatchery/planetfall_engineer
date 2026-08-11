use super::*;

#[test]
fn every_visible_build_control_maps_to_its_action() {
    assert_eq!(
        build_click_at(1040.0, 337.0),
        Some(BuildClick::Palette(DeviceId::Channel))
    );
    assert_eq!(
        build_click_at(1200.0, 441.0),
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
}
