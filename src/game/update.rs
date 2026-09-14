//! Frame input routing and fixed-tick advancement for the active operation.

use super::{FrontendMode, Game, VerificationMode};
use crate::{
    devices::DeviceId,
    mission::{campaign_summary, CommandKind, MissionPhase},
    simulation::{FluidId, TerrainAction},
    state::{save_session, TimeControl},
    ui,
    ui_action::{pause_action_at, UiAction},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::virtual_mouse_position;

impl Game {
    pub fn update(&mut self, dt: f32) {
        if self.frontend_mode != FrontendMode::Playing {
            self.update_frontend();
            return;
        }
        if self.session.mission.phase == MissionPhase::Briefing {
            self.update_briefing_input();
            return;
        }
        if matches!(
            self.session.mission.phase,
            MissionPhase::Success | MissionPhase::Failure
        ) && self.handle_terminal_click()
        {
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.dispatch_action(UiAction::TogglePause);
        }
        if self.pause_menu {
            self.update_pause_input();
            return;
        }
        self.update_camera_and_pointer(dt);
        self.update_shortcuts();
        self.advance_simulation(dt);
    }

    fn update_briefing_input(&mut self) {
        if is_key_pressed(KeyCode::Enter) || self.handle_briefing_click() {
            self.dispatch_action(UiAction::BeginMission);
        }
    }

    fn update_pause_input(&mut self) {
        let action = if is_mouse_button_pressed(MouseButton::Left) {
            let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
            pause_action_at(point.x, point.y)
        } else if is_key_pressed(KeyCode::F12) {
            Some(UiAction::ResetMission)
        } else if is_key_pressed(KeyCode::F5) {
            Some(UiAction::Save)
        } else if is_key_pressed(KeyCode::F9) {
            Some(UiAction::Load)
        } else {
            None
        };
        if let Some(action) = action {
            self.dispatch_action(action);
            if matches!(action, UiAction::SetTime(_) | UiAction::ResetMission) {
                self.pause_menu = false;
            }
        }
    }

    fn update_camera_and_pointer(&mut self, dt: f32) {
        if self.camera.update(
            dt,
            self.session.simulation.width as usize,
            self.session.simulation.height as usize,
        ) {
            let _ = self.session.mission.admit(CommandKind::Camera);
        }
        if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let drag = mouse_delta_position();
            if (82.0..=604.0).contains(&mouse_y) && mouse_x < 1_000.0 && drag.length_squared() > 0.0
            {
                self.camera.target.x = (self.camera.target.x - drag.x * 0.03)
                    .clamp(0.0, self.session.simulation.width as f32);
                self.camera.target.y = (self.camera.target.y - drag.y * 0.03)
                    .clamp(0.0, self.session.simulation.height as f32);
                let _ = self.session.mission.admit(CommandKind::Camera);
            }
        }
        let verification_control_claimed = self.handle_verification_click();
        if is_mouse_button_pressed(MouseButton::Left)
            && !self.handle_tutorial_prompt_click()
            && !self.handle_palette_click()
            && !self.handle_build_action_click()
            && !self.handle_time_click()
            && !self.handle_field_control_click()
            && !verification_control_claimed
        {
            self.select_from_pointer();
        }
    }

    fn update_shortcuts(&mut self) {
        self.update_time_and_selection_keys();
        self.update_terrain_keys();
        self.update_verification_keys();
        self.update_device_keys();
        self.update_persistence_keys();
    }

    fn update_time_and_selection_keys(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.dispatch_action(UiAction::SetTime(match self.session.time_control {
                TimeControl::Paused => TimeControl::OneX,
                TimeControl::OneX => TimeControl::Paused,
                _ => TimeControl::Paused,
            }));
        }
        for (key, time) in [
            (KeyCode::Key1, TimeControl::OneX),
            (KeyCode::Key2, TimeControl::TwoX),
            (KeyCode::Key4, TimeControl::FourX),
        ] {
            if is_key_pressed(key) {
                self.dispatch_action(UiAction::SetTime(time));
            }
        }
        for (key, delta) in [
            (KeyCode::Up, (0, -1)),
            (KeyCode::Down, (0, 1)),
            (KeyCode::Left, (-1, 0)),
            (KeyCode::Right, (1, 0)),
        ] {
            if is_key_pressed(key) {
                self.dispatch_action(UiAction::MoveSelection(delta.0, delta.1));
            }
        }
    }

    fn update_terrain_keys(&mut self) {
        for (key, action) in [
            (KeyCode::X, UiAction::Terrain(TerrainAction::Excavate)),
            (KeyCode::R, UiAction::Terrain(TerrainAction::Raise)),
            (KeyCode::T, UiAction::Terrain(TerrainAction::Seal)),
            (KeyCode::I, UiAction::Inspect),
            (KeyCode::L, UiAction::Inject(FluidId::Lava)),
            (KeyCode::G, UiAction::Inject(FluidId::ToxicSlurry)),
            (KeyCode::Y, UiAction::CycleOverlay),
        ] {
            if is_key_pressed(key) {
                self.dispatch_action(action);
            }
        }
    }

    fn update_verification_keys(&mut self) {
        if is_key_pressed(KeyCode::F1) {
            self.dispatch_action(UiAction::ToggleLab);
        }
        if is_key_pressed(KeyCode::F2) {
            self.dispatch_action(if self.verification_mode.is_some() {
                UiAction::RestoreCampaign
            } else {
                UiAction::EnterShowcase(DeviceId::Channel)
            });
        }
        if is_key_pressed(KeyCode::V) {
            if let Some(VerificationMode::Showcase(current)) = self.verification_mode {
                let next = DeviceId::ALL[(DeviceId::ALL
                    .iter()
                    .position(|device| *device == current)
                    .unwrap_or(0)
                    + 1)
                    % DeviceId::ALL.len()];
                self.dispatch_action(UiAction::EnterShowcase(next));
            }
        }
        if is_key_pressed(KeyCode::F3) {
            self.notice = campaign_summary();
        }
        if is_key_pressed(KeyCode::N) {
            self.dispatch_action(UiAction::NextCampaign);
        }
        if is_key_pressed(KeyCode::F4) {
            self.dispatch_action(UiAction::SkipTutorial);
        }
    }

    fn update_device_keys(&mut self) {
        for (key, action) in [
            (KeyCode::C, UiAction::RemoveSelectedDevice),
            (KeyCode::B, UiAction::QueueDevice(DeviceId::Channel)),
            (KeyCode::P, UiAction::QueueDevice(DeviceId::Pipe)),
            (KeyCode::O, UiAction::QueueDevice(DeviceId::Pump)),
            (KeyCode::F, UiAction::QueueDevice(DeviceId::Floodgate)),
            (KeyCode::Z, UiAction::RotatePlacement),
            (KeyCode::J, UiAction::SetGate(0)),
            (KeyCode::K, UiAction::SetGate(5_000)),
            (KeyCode::H, UiAction::SetGate(10_000)),
        ] {
            if is_key_pressed(key) {
                self.dispatch_action(action);
            }
        }
        if is_key_pressed(KeyCode::Enter) {
            self.dispatch_action(if self.session.simulation.devices.queued.is_empty() {
                UiAction::DismissPrompt
            } else {
                UiAction::CommitPlan
            });
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.dispatch_action(UiAction::CancelPlan);
        }
    }

    fn update_persistence_keys(&mut self) {
        if is_key_pressed(KeyCode::F5) {
            self.dispatch_action(UiAction::Save);
        }
        if is_key_pressed(KeyCode::F9) {
            self.dispatch_action(UiAction::Load);
        }
        if is_key_pressed(KeyCode::F6) {
            self.dispatch_action(if self.verification_mode.is_some() {
                UiAction::ResetVerification
            } else {
                UiAction::Checkpoint
            });
        }
        if is_key_pressed(KeyCode::F7) {
            self.dispatch_action(UiAction::FailMission);
        }
        if is_key_pressed(KeyCode::F8) {
            self.dispatch_action(if self.verification_mode.is_some() {
                UiAction::StepVerification
            } else {
                UiAction::RunShowcaseReport
            });
        }
        if is_key_pressed(KeyCode::F10) {
            self.dispatch_action(UiAction::ReloadMission);
        }
        if is_key_pressed(KeyCode::F11) {
            self.dispatch_action(UiAction::RunScenarioReport);
        }
        if is_key_pressed(KeyCode::F12) {
            self.dispatch_action(UiAction::ResetMission);
        }
    }

    fn advance_simulation(&mut self, dt: f32) {
        let ticks = self.session.update(dt);
        if ticks == 0 {
            return;
        }
        self.session.mission.on_tick(&self.session.simulation);
        if self.session.mission.phase != MissionPhase::Success {
            return;
        }
        self.session
            .campaign
            .record_success(self.session.mission.id, self.session.mission.tick);
        self.session.time_control = TimeControl::Paused;
        self.notice = save_session(&self.session, &self.data.config)
            .map(|_| {
                format!(
                    "Mission success — {} complete and campaign progress saved",
                    self.mission_title(self.session.mission.id)
                )
            })
            .unwrap_or_else(|error| {
                format!("Mission complete; campaign autosave needs attention: {error}")
            });
    }
}
