use super::*;

#[test]
fn title_buttons_have_distinct_click_targets() {
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::Title, 640.0, 324.0),
        Some(0)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::Title, 640.0, 386.0),
        Some(1)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::Title, 640.0, 448.0),
        Some(2)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::Title, 100.0, 324.0),
        None
    );
}

#[test]
fn verification_buttons_select_lab_and_each_device_bay() {
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 640.0, 324.0),
        Some(0)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 460.0, 387.0),
        Some(1)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 800.0, 387.0),
        Some(2)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 460.0, 579.0),
        Some(9)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 800.0, 579.0),
        Some(10)
    );
    assert_eq!(
        frontend_mouse_choice_at(FrontendMode::VerificationSelect, 640.0, 620.0),
        None
    );
}
