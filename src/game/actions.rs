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
        match action {
            UiAction::TogglePause => {
                self.pause_menu = !self.pause_menu;
                if self.pause_menu {
                    self.session.time_control = TimeControl::Paused;
                    self.notice = "Pause menu open".into();
                }
                true
            }
            UiAction::SetTime(time) => {
                self.set_time(time);
                true
            }
            UiAction::Select(position) => {
                if self.session.simulation.index(position).is_none() {
                    return false;
                }
                self.session.selected = position;
                let _ = self.session.mission.admit(CommandKind::Select);
                self.notice = format!("Survey target selected: {}, {}", position.x, position.y);
                true
            }
            UiAction::MoveSelection(dx, dy) => {
                self.session.move_selected(dx, dy);
                let position = self.session.selected;
                let _ = self.session.mission.admit(CommandKind::Select);
                self.notice = format!("Survey target selected: {}, {}", position.x, position.y);
                true
            }
            UiAction::Inspect => {
                if self.admit(CommandKind::Inspect) {
                    self.notice = format!(
                        "Inspecting cell {}, {}",
                        self.session.selected.x, self.session.selected.y
                    );
                }
                true
            }
            UiAction::Terrain(action) => {
                self.apply_terrain(action);
                true
            }
            UiAction::QueueDevice(device) => {
                self.queue_device(device);
                true
            }
            UiAction::RotatePlacement => {
                self.placement_rotation = (self.placement_rotation + 1) % 4;
                self.notice = format!("Placement rotation {}°", self.placement_rotation * 90);
                true
            }
            UiAction::CommitPlan => {
                self.commit_build_plan();
                true
            }
            UiAction::CancelPlan => {
                self.cancel_build_plan();
                true
            }
            UiAction::SetGate(setting) => {
                self.set_gate(setting);
                true
            }
            UiAction::RecoverDeposit => self.recover_selected_deposit(),
            UiAction::ToggleSelectedDevice => self.toggle_selected_device(),
            UiAction::Save => {
                self.notice = save_session(&self.session, &self.data.config)
                    .map(|_| "Checkpoint saved".into())
                    .unwrap_or_else(|error| error);
                true
            }
            UiAction::Load => {
                match load_session(&self.data.config) {
                    Ok(session) => {
                        self.session = session;
                        self.notice = "Checkpoint loaded".into();
                    }
                    Err(error) => self.notice = error,
                }
                true
            }
            UiAction::ResetMission => {
                self.reset_mission();
                true
            }
            UiAction::BeginMission => {
                if self.session.mission.phase == MissionPhase::Briefing {
                    self.session.mission.start();
                    self.notice = format!(
                        "{} operation active — follow the field guide and begin when ready",
                        self.session.mission.id.name()
                    );
                }
                true
            }
            UiAction::SkipTutorial => {
                self.skip_tutorial();
                true
            }
            UiAction::ToggleLab => {
                self.toggle_lab_mode();
                true
            }
            UiAction::EnterShowcase(device) => {
                self.enter_showcase(device);
                true
            }
            UiAction::RestoreCampaign => {
                self.restore_campaign_session();
                true
            }
            UiAction::StepVerification => {
                self.step_verification();
                true
            }
            UiAction::NextCampaign => {
                self.select_next_campaign();
                true
            }
            UiAction::StartNewCampaign => {
                self.start_new_campaign();
                true
            }
            UiAction::OpenCampaignSelect => {
                self.frontend_mode = crate::game::FrontendMode::CampaignSelect;
                true
            }
            UiAction::OpenVerificationSelect => {
                self.frontend_mode = crate::game::FrontendMode::VerificationSelect;
                true
            }
            UiAction::SelectMission(id) => {
                if self.session.campaign.unlocked[id.sequence() - 1] {
                    self.load_mission(id, "selected from campaign board");
                } else {
                    self.notice = format!(
                        "{} is locked — finish earlier campaign work first",
                        id.name()
                    );
                }
                true
            }
            UiAction::TerminalPrimary => {
                if self.session.mission.phase == MissionPhase::Success {
                    self.select_next_campaign();
                } else {
                    self.reset_mission();
                }
                true
            }
            UiAction::TerminalSecondary => {
                if self.session.mission.phase == MissionPhase::Success {
                    self.frontend_mode = crate::game::FrontendMode::CampaignSelect;
                    self.notice = "Campaign board opened after mission success".into();
                } else {
                    self.checkpoint_session = None;
                    self.load_mission(self.session.mission.id, "restarted from failure");
                }
                true
            }
            UiAction::ResetVerification => {
                self.reset_verification();
                true
            }
            UiAction::RemoveSelectedDevice => self.remove_selected_device(),
            UiAction::Checkpoint => {
                self.session.mission.checkpoint();
                self.checkpoint_session = Some(self.session.clone());
                self.notice = format!(
                    "Mission checkpoint recorded at tick {}",
                    self.session.mission.checkpoint_tick
                );
                true
            }
            UiAction::FailMission => {
                self.session.mission.fail("manual failure-path check");
                self.notice = "Mission failed — reset to checkpoint".into();
                true
            }
            UiAction::ReloadMission => {
                self.load_mission(self.session.mission.id, "reloaded from authored state");
                true
            }
            UiAction::Inject(fluid) => {
                self.session
                    .simulation
                    .inject(self.session.selected, fluid, 500);
                true
            }
            UiAction::CycleOverlay => {
                self.overlay_mode = (self.overlay_mode + 1) % 5;
                self.notice = format!("{} overlay", crate::game::overlay_name(self.overlay_mode));
                true
            }
            UiAction::RunShowcaseReport => {
                let admission = self.session.mission.admit(CommandKind::DismissPrompt);
                self.notice = format!(
                    "{} — tutorial command: {admission:?}",
                    crate::devices::run_all_showcases()
                );
                true
            }
            UiAction::RunScenarioReport => {
                self.notice = run_all_scenarios();
                true
            }
            UiAction::DismissPrompt => {
                let _ = self.admit(CommandKind::DismissPrompt);
                true
            }
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
