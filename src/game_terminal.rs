//! Touch actions for mission success and failure panels.

use crate::{
    game::{FrontendMode, Game},
    mission::MissionPhase,
    ui,
};
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
            Some(TerminalAction::Primary) if success => self.select_next_campaign(),
            Some(TerminalAction::Secondary) if success => {
                self.frontend_mode = FrontendMode::CampaignSelect;
                self.notice = "Campaign board opened after mission success".into();
            }
            Some(TerminalAction::Primary) => self.reset_mission(),
            Some(TerminalAction::Secondary) => {
                self.checkpoint_session = None;
                self.load_mission(self.session.mission.id, "restarted from failure");
            }
            None => return false,
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalAction {
    Primary,
    Secondary,
}

fn terminal_action_at(x: f32, y: f32) -> Option<TerminalAction> {
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

#[cfg(test)]
mod tests;
