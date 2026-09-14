//! Touch actions for mission success and failure panels.

use crate::{game::Game, mission::MissionPhase, ui, ui_action::UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::virtual_mouse_position;

impl Game {
    pub(crate) fn handle_terminal_click(&mut self) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let success = self.session.mission.phase == MissionPhase::Success;
        match terminal_action_at(point.x, point.y) {
            Some(TerminalAction::Primary) if success => {
                self.dispatch_action(UiAction::TerminalPrimary);
            }
            Some(TerminalAction::Secondary) if success => {
                self.dispatch_action(UiAction::TerminalSecondary);
            }
            Some(TerminalAction::Primary) => {
                self.dispatch_action(UiAction::TerminalPrimary);
            }
            Some(TerminalAction::Secondary) => {
                self.dispatch_action(UiAction::TerminalSecondary);
            }
            None => return false,
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalAction {
    Primary,
    Secondary,
}

pub fn terminal_action_at(x: f32, y: f32) -> Option<TerminalAction> {
    if !(374.0..=414.0).contains(&y) {
        return None;
    }
    if (336.0..=620.0).contains(&x) {
        Some(TerminalAction::Primary)
    } else if (660.0..=944.0).contains(&x) {
        Some(TerminalAction::Secondary)
    } else {
        None
    }
}
