//! Player-facing laboratory and device-showcase transitions.

use crate::devices::{showcase_world, DeviceId};
use crate::game::{FoundationCamera, Game, VerificationMode};
use crate::mission::{MissionId, MissionState};
use crate::state::{CellPos, GameSession, TimeControl, WorldState};
use crate::ui;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationClick {
    Reset,
    Step,
    Return,
}

impl Game {
    pub(crate) fn handle_verification_click(&mut self) -> bool {
        if self.verification_mode.is_none() || !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        let (mouse_x, mouse_y) = mouse_position();
        let x = mouse_x / (screen_width() / ui::LOGICAL_WIDTH);
        let y = mouse_y / (screen_height() / ui::LOGICAL_HEIGHT);
        match verification_click_at(x, y) {
            Some(VerificationClick::Reset) => self.reset_verification(),
            Some(VerificationClick::Step) => self.step_verification(),
            Some(VerificationClick::Return) => self.restore_campaign_session(),
            None => return false,
        }
        true
    }

    pub(crate) fn toggle_lab_mode(&mut self) {
        if let Some(campaign) = self.saved_campaign_session.take() {
            if self.verification_returns_to_menu {
                self.verification_mode = None;
                self.verification_returns_to_menu = false;
                self.frontend_mode = crate::game::FrontendMode::VerificationSelect;
                self.notice = "Returned to verification grounds".into();
                return;
            }
            self.restore_campaign(campaign);
            return;
        }
        let campaign = self.session.clone();
        let report = self.lab.automatic_scenario();
        self.session.simulation = self.lab.world.clone();
        self.session.world = world_state_for(&self.session.simulation);
        self.session.mission = MissionState::new(MissionId::L02HoldingLine);
        self.session.mission.start();
        self.session.tick = self.session.simulation.tick;
        self.session.selected = CellPos {
            x: self.session.simulation.width / 2,
            y: self.session.simulation.height / 2,
        };
        self.session.time_control = TimeControl::Paused;
        self.saved_campaign_session = Some(campaign);
        self.verification_mode = Some(VerificationMode::Lab);
        self.camera = FoundationCamera::new(
            self.session.simulation.width as usize,
            self.session.simulation.height as usize,
        );
        self.notice = format!(
            "lab_fluids_all {} — F1 return — tick {} hash {:016X}",
            if report.passed { "PASS" } else { "FAIL" },
            report.tick,
            report.state_hash
        );
    }

    pub(crate) fn enter_showcase(&mut self, device: DeviceId) {
        if self.saved_campaign_session.is_none() {
            self.saved_campaign_session = Some(self.session.clone());
        }
        self.session.simulation = showcase_world(device);
        self.session.world = world_state_for(&self.session.simulation);
        self.session.mission = MissionState::new(MissionId::L02HoldingLine);
        self.session.mission.start();
        self.session.tick = self.session.simulation.tick;
        self.session.selected = CellPos { x: 16, y: 9 };
        self.session.time_control = TimeControl::Paused;
        self.verification_mode = Some(VerificationMode::Showcase(device));
        self.camera = FoundationCamera::new(32, 18);
        self.notice = format!("device_{} — F2 return — V next showcase", device.name());
    }

    pub(crate) fn restore_campaign_session(&mut self) {
        if let Some(campaign) = self.saved_campaign_session.take() {
            if self.verification_returns_to_menu {
                self.verification_mode = None;
                self.verification_returns_to_menu = false;
                self.frontend_mode = crate::game::FrontendMode::VerificationSelect;
                self.notice = "Returned to verification grounds".into();
                return;
            }
            self.restore_campaign(campaign);
        }
    }

    pub(crate) fn reset_verification(&mut self) {
        match self.verification_mode {
            Some(VerificationMode::Lab) => {
                self.lab.reset();
                self.session.simulation = self.lab.world.clone();
            }
            Some(VerificationMode::Showcase(device)) => {
                self.session.simulation = showcase_world(device);
            }
            None => return,
        }
        self.session.world = world_state_for(&self.session.simulation);
        self.session.tick = self.session.simulation.tick;
        self.session.time_control = TimeControl::Paused;
        self.notice = "Verification map reset — F8 advances one tick".into();
    }

    pub(crate) fn step_verification(&mut self) {
        if self.verification_mode.is_none() {
            return;
        }
        self.session.tick();
        self.session.mission.on_tick(&self.session.simulation);
        self.notice = format!(
            "Verification tick {} — balance {:+} vU",
            self.session.simulation.tick,
            self.session.simulation.mass_balance_error()
        );
    }

    fn restore_campaign(&mut self, campaign: GameSession) {
        self.session = campaign;
        self.verification_mode = None;
        self.camera = FoundationCamera::new(
            self.session.simulation.width as usize,
            self.session.simulation.height as usize,
        );
        self.notice = "Returned to campaign session".into();
    }
}

fn verification_click_at(x: f32, y: f32) -> Option<VerificationClick> {
    if !(1018.0..1248.0).contains(&x) || !(532.0..560.0).contains(&y) {
        return None;
    }
    if x < 1092.0 {
        Some(VerificationClick::Reset)
    } else if x < 1170.0 {
        Some(VerificationClick::Step)
    } else {
        Some(VerificationClick::Return)
    }
}

fn world_state_for(simulation: &crate::simulation::SimulationWorld) -> WorldState {
    let mut world = WorldState::new(simulation.width as usize, simulation.height as usize);
    for (cell, sim_cell) in world.cells.iter_mut().zip(&simulation.cells) {
        cell.height_hu = sim_cell.height_hu;
        cell.sealed = sim_cell.sealed;
    }
    world
}

#[cfg(test)]
mod tests {
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
}
