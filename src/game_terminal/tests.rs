use super::*;

#[test]
fn terminal_buttons_have_distinct_touch_targets() {
    assert_eq!(
        terminal_action_at(480.0, 394.0),
        Some(TerminalAction::Primary)
    );
    assert_eq!(
        terminal_action_at(800.0, 394.0),
        Some(TerminalAction::Secondary)
    );
    assert_eq!(terminal_action_at(640.0, 394.0), None);
}
