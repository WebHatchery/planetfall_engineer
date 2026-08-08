//! Mouse and keyboard build-palette actions.

use crate::game::Game;
use crate::state::TimeControl;
use crate::{devices::DeviceId, ui};
use macroquad::prelude::*;

impl Game {
    pub(crate) fn handle_palette_click(&mut self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        let scale_x = screen_width() / ui::LOGICAL_WIDTH;
        let scale_y = screen_height() / ui::LOGICAL_HEIGHT;
        let x = mouse_x / scale_x;
        let y = mouse_y / scale_y;
        if !(1018.0..1248.0).contains(&x) || !(326.0..456.0).contains(&y) {
            return false;
        }
        let column = if x < 1132.0 { 0 } else { 1 };
        let row = ((y - 326.0) / 26.0) as usize;
        let index = row * 2 + column;
        if let Some(device) = DeviceId::ALL.get(index).copied() {
            self.queue_device(device);
        }
        true
    }

    pub(crate) fn handle_build_action_click(&mut self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        let scale_x = screen_width() / ui::LOGICAL_WIDTH;
        let scale_y = screen_height() / ui::LOGICAL_HEIGHT;
        let x = mouse_x / scale_x;
        let y = mouse_y / scale_y;
        if !(1018.0..1248.0).contains(&x) || !(468.0..494.0).contains(&y) {
            return false;
        }
        if x < 1092.0 {
            self.placement_rotation = (self.placement_rotation + 1) % 4;
            self.notice = format!("Placement rotation {}°", self.placement_rotation * 90);
        } else if x < 1170.0 {
            self.commit_build_plan();
        } else {
            self.cancel_build_plan();
        }
        true
    }

    pub(crate) fn handle_time_click(&mut self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        let scale_x = screen_width() / ui::LOGICAL_WIDTH;
        let scale_y = screen_height() / ui::LOGICAL_HEIGHT;
        let x = mouse_x / scale_x;
        let y = mouse_y / scale_y;
        if !(1018.0..1250.0).contains(&x) || !(496.0..522.0).contains(&y) {
            return false;
        }
        let time = if x < 1076.0 {
            TimeControl::Paused
        } else if x < 1136.0 {
            TimeControl::OneX
        } else if x < 1196.0 {
            TimeControl::TwoX
        } else {
            TimeControl::FourX
        };
        self.set_time(time);
        true
    }

    pub(crate) fn queue_device(&mut self, device: DeviceId) {
        self.placement_device = device;
        if !self.admit(crate::mission::CommandKind::QueueDevice(device)) {
            return;
        }
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let result = devices.queue(
            &self.session.simulation,
            device,
            self.session.selected,
            self.placement_rotation,
            self.session.mission.budget,
        );
        self.session.simulation.devices = devices;
        self.notice = result
            .map(|id| {
                format!(
                    "Queued {} plan #{id} at {}° — Enter commit, Backspace cancel",
                    device.name(),
                    self.placement_rotation * 90
                )
            })
            .unwrap_or_else(|error| format!("Queue rejected: {error:?}"));
    }

    pub(crate) fn commit_build_plan(&mut self) {
        if !self.admit(crate::mission::CommandKind::CommitPlan) {
            return;
        }
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let result = devices.commit_plan(&self.session.simulation, self.session.mission.budget);
        self.session.simulation.devices = devices;
        self.notice = result
            .map(|entities| format!("Committed {} build plan(s)", entities.len()))
            .unwrap_or_else(|error| format!("Plan commit rejected: {error:?}"));
    }

    pub(crate) fn cancel_build_plan(&mut self) {
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let cancelled = devices.cancel_last_plan();
        self.session.simulation.devices = devices;
        self.notice = if cancelled {
            "Last build plan cancelled"
        } else {
            "No queued build plan"
        }
        .into();
    }
}
