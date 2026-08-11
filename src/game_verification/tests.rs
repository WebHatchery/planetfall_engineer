use super::*;

#[test]
fn verification_controls_have_distinct_touch_targets() {
    assert_eq!(
        verification_click_at(1040.0, 546.0),
        Some(VerificationClick::Reset)
    );
    assert_eq!(
        verification_click_at(1120.0, 546.0),
        Some(VerificationClick::Step)
    );
    assert_eq!(
        verification_click_at(1200.0, 546.0),
        Some(VerificationClick::Return)
    );
    assert_eq!(verification_click_at(1120.0, 570.0), None);
}
