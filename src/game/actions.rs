//! Central dispatcher for UI intents that change game, mission, or save state.

use super::Game;
use crate::{
    mission::{CommandKind, MissionPhase},
    replay::run_all_scenarios,
    state::{load_session, save_session, TimeControl},
    ui_action::UiAction,
};

impl Game {
    pub(crate) fn dispatch_action(&mut self, action: UiAction) -> bool {
        self.dispatch_field_action(action)
            .or_else(|| self.dispatch_session_action(action))
            .or_else(|| self.dispatch_menu_action(action))
            .or_else(|| self.dispatch_debug_action(action))
            .unwrap_or(false)
    }

    fn dispatch_field_action(&mut self, action: UiAction) -> Option<bool> {
        match action {
            UiAction::TogglePause => {
                self.pause_menu = !self.pause_menu;
                if self.pause_menu {
                    self.session.time_control = TimeControl::Paused;
                    self.notice = "Pause menu open".into();
                }
                Some(true)
            }
            UiAction::SetTime(time) => {
                self.set_time(time);
                Some(true)
            }
            UiAction::Select(position) => {
                if self.session.simulation.index(position).is_none() {
                    return Some(false);
                }
                self.session.selected = position;
                let _ = self.session.mission.admit(CommandKind::Select);
                self.notice = format!("Survey target selected: {}, {}", position.x, position.y);
                Some(true)
            }
            UiAction::MoveSelection(dx, dy) => {
                self.session.move_selected(dx, dy);
                let position = self.session.selected;
                let _ = self.session.mission.admit(CommandKind::Select);
                self.notice = format!("Survey target selected: {}, {}", position.x, position.y);
                Some(true)
            }
            UiAction::Inspect => {
                if self.admit(CommandKind::Inspect) {
                    self.notice = format!(
                        "Inspecting cell {}, {}",
                        self.session.selected.x, self.session.selected.y
                    );
                }
                Some(true)
            }
            UiAction::Terrain(action) => {
                self.apply_terrain(action);
                Some(true)
            }
            UiAction::QueueDevice(device) => {
                self.queue_device(device);
                Some(true)
            }
            UiAction::RotatePlacement => {
                self.placement_rotation = (self.placement_rotation + 1) % 4;
                self.notice = format!("Placement rotation {}°", self.placement_rotation * 90);
                Some(true)
            }
            UiAction::CommitPlan => {
                self.commit_build_plan();
                Some(true)
            }
            UiAction::CancelPlan => {
                self.cancel_build_plan();
                Some(true)
            }
            UiAction::SetGate(setting) => {
                self.set_gate(setting);
                Some(true)
            }
            UiAction::RecoverDeposit => Some(self.recover_selected_deposit()),
            UiAction::ToggleSelectedDevice => Some(self.toggle_selected_device()),
            _ => None,
        }
    }

    fn dispatch_session_action(&mut self, action: UiAction) -> Option<bool> {
        match action {
            UiAction::Save => {
                self.notice = save_session(&self.session, &self.data.config)
                    .map(|_| "Checkpoint saved".into())
                    .unwrap_or_else(|error| error);
                Some(true)
            }
            UiAction::Load => {
                match load_session(&self.data.config) {
                    Ok(session) => {
                        self.session = session;
                        self.notice = "Checkpoint loaded".into();
                    }
                    Err(error) => self.notice = error,
                }
                Some(true)
            }
            UiAction::ResetMission => {
                self.reset_mission();
                Some(true)
            }
            UiAction::BeginMission => {
                if self.session.mission.phase == MissionPhase::Briefing {
                    self.session.mission.start();
                    self.notice = format!(
                        "{} operation active — follow the field guide and begin when ready",
                        self.mission_title(self.session.mission.id)
                    );
                }
                Some(true)
            }
            UiAction::SkipTutorial => {
                self.skip_tutorial();
                Some(true)
            }
            UiAction::ToggleLab => {
                self.toggle_lab_mode();
                Some(true)
            }
            UiAction::EnterShowcase(device) => {
                self.enter_showcase(device);
                Some(true)
            }
            UiAction::RestoreCampaign => {
                self.restore_campaign_session();
                Some(true)
            }
            UiAction::StepVerification => {
                self.step_verification();
                Some(true)
            }
            UiAction::NextCampaign => {
                self.select_next_campaign();
                Some(true)
            }
            _ => None,
        }
    }

    fn dispatch_menu_action(&mut self, action: UiAction) -> Option<bool> {
        match action {
            UiAction::StartNewCampaign => {
                self.start_new_campaign();
                Some(true)
            }
            UiAction::OpenCampaignSelect => {
                self.frontend_mode = crate::game::FrontendMode::CampaignSelect;
                Some(true)
            }
            UiAction::OpenVerificationSelect => {
                self.frontend_mode = crate::game::FrontendMode::VerificationSelect;
                Some(true)
            }
            UiAction::SelectMission(id) => {
                if self.session.campaign.unlocked[id.sequence() - 1] {
                    self.load_mission(id, "selected from campaign board");
                } else {
                    self.notice = format!(
                        "{} is locked — finish earlier campaign work first",
                        self.mission_title(id)
                    );
                }
                Some(true)
            }
            UiAction::TerminalPrimary => {
                if self.session.mission.phase == MissionPhase::Success {
                    self.select_next_campaign();
                } else {
                    self.reset_mission();
                }
                Some(true)
            }
            UiAction::TerminalSecondary => {
                if self.session.mission.phase == MissionPhase::Success {
                    self.frontend_mode = crate::game::FrontendMode::CampaignSelect;
                    self.notice = "Campaign board opened after mission success".into();
                } else {
                    self.checkpoint_session = None;
                    self.load_mission(self.session.mission.id, "restarted from failure");
                }
                Some(true)
            }
            UiAction::ResetVerification => {
                self.reset_verification();
                Some(true)
            }
            UiAction::RemoveSelectedDevice => Some(self.remove_selected_device()),
            _ => None,
        }
    }

    fn dispatch_debug_action(&mut self, action: UiAction) -> Option<bool> {
        match action {
            UiAction::Checkpoint => {
                self.session.mission.checkpoint();
                self.checkpoint_session = Some(self.session.clone());
                self.notice = format!(
                    "Mission checkpoint recorded at tick {}",
                    self.session.mission.checkpoint_tick
                );
                Some(true)
            }
            UiAction::FailMission => {
                self.session.mission.fail("manual failure-path check");
                self.notice = "Mission failed — reset to checkpoint".into();
                Some(true)
            }
            UiAction::ReloadMission => {
                self.load_mission(self.session.mission.id, "reloaded from authored state");
                Some(true)
            }
            UiAction::Inject(fluid) => {
                self.session
                    .simulation
                    .inject(self.session.selected, fluid, 500);
                Some(true)
            }
            UiAction::CycleOverlay => {
                self.overlay_mode = (self.overlay_mode + 1) % 5;
                self.notice = format!("{} overlay", crate::game::overlay_name(self.overlay_mode));
                Some(true)
            }
            UiAction::RunShowcaseReport => {
                let admission = self.session.mission.admit(CommandKind::DismissPrompt);
                self.notice = format!(
                    "{} — tutorial command: {admission:?}",
                    crate::devices::run_all_showcases()
                );
                Some(true)
            }
            UiAction::RunScenarioReport => {
                self.notice = run_all_scenarios();
                Some(true)
            }
            UiAction::DismissPrompt => {
                let _ = self.admit(CommandKind::DismissPrompt);
                Some(true)
            }
            _ => None,
        }
    }

    fn recover_selected_deposit(&mut self) -> bool {
        if !self.admit(CommandKind::RecoverDeposit) {
            return true;
        }
        self.notice = self
            .session
            .simulation
            .recover_deposit_at(self.session.selected)
            .map(|yield_fu| format!("Recovered {} fabU from local deposit", yield_fu))
            .unwrap_or_else(|error| format!("Deposit recovery rejected: {error}"));
        true
    }

    fn toggle_selected_device(&mut self) -> bool {
        if !self.admit(CommandKind::ToggleDevice) {
            return true;
        }
        let Some(entity_id) = self
            .session
            .simulation
            .devices
            .devices
            .iter()
            .find(|device| device.anchor == self.session.selected && !device.authored)
            .map(|device| device.entity_id)
        else {
            self.notice = "Select a player device to toggle".into();
            return true;
        };
        let enabled = self
            .session
            .simulation
            .devices
            .devices
            .iter()
            .find(|device| device.entity_id == entity_id)
            .is_some_and(|device| !device.enabled);
        self.session
            .simulation
            .devices
            .set_enabled(entity_id, enabled);
        self.notice = if enabled {
            "Selected device enabled".into()
        } else {
            "Selected device disabled".into()
        };
        true
    }

    fn remove_selected_device(&mut self) -> bool {
        let Some(entity_id) = self
            .session
            .simulation
            .devices
            .devices
            .iter()
            .find(|device| device.anchor == self.session.selected)
            .map(|device| device.entity_id)
        else {
            self.notice = "Select a device to remove".into();
            return true;
        };
        let mut fabrication = std::mem::take(&mut self.session.simulation.fabrication);
        let removed = self
            .session
            .simulation
            .devices
            .remove(entity_id, &mut fabrication);
        self.session.simulation.fabrication = fabrication;
        self.notice = if removed {
            "Device removed and fabrication refunded".into()
        } else {
            "Device removal rejected".into()
        };
        true
    }
}
