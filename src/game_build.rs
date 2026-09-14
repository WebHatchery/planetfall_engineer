//! Mouse and keyboard build-palette actions.

use crate::game::Game;
use crate::state::TimeControl;
use crate::{devices::DeviceId, ui, ui_action::UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::virtual_mouse_position;

impl Game {
    pub(crate) fn handle_palette_click(&mut self) -> bool {
        if let Some(BuildClick::Palette(device)) = mouse_build_click() {
            self.dispatch_action(UiAction::QueueDevice(device));
            return true;
        }
        false
    }

    pub(crate) fn handle_build_action_click(&mut self) -> bool {
        let Some(click) = mouse_build_click() else {
            return false;
        };
        match click {
            BuildClick::Rotate | BuildClick::Commit | BuildClick::Cancel => {
                self.dispatch_action(build_click_action(click));
                true
            }
            _ => false,
        }
    }

    pub(crate) fn handle_time_click(&mut self) -> bool {
        if let Some(BuildClick::Time(time)) = mouse_build_click() {
            self.dispatch_action(UiAction::SetTime(time));
            true
        } else {
            false
        }
    }

    pub(crate) fn handle_field_control_click(&mut self) -> bool {
        let Some(control) = field_control_click() else {
            return false;
        };
        self.dispatch_action(field_control_action(control));
        true
    }

    pub(crate) fn queue_device(&mut self, device: DeviceId) {
        self.placement_device = device;
        if !self.admit(crate::mission::CommandKind::QueueDevice(device)) {
            return;
        }
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let mut fabrication = std::mem::take(&mut self.session.simulation.fabrication);
        let result = devices.queue(
            &self.session.simulation,
            device,
            self.session.selected,
            self.placement_rotation,
            &mut fabrication,
        );
        self.session.simulation.devices = devices;
        self.session.simulation.fabrication = fabrication;
        self.notice = result
            .map(|id| {
                format!(
                    "Queued {} plan #{id} at {}° — use COMMIT or CANCEL",
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
        let mut fabrication = std::mem::take(&mut self.session.simulation.fabrication);
        let result = devices.commit_plan(&self.session.simulation, &mut fabrication);
        self.session.simulation.devices = devices;
        self.session.simulation.fabrication = fabrication;
        self.notice = result
            .map(|entities| format!("Committed {} build plan(s)", entities.len()))
            .unwrap_or_else(|error| format!("Plan commit rejected: {error:?}"));
    }

    pub(crate) fn cancel_build_plan(&mut self) {
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let mut fabrication = std::mem::take(&mut self.session.simulation.fabrication);
        let cancelled = devices.cancel_last_plan(&mut fabrication);
        self.session.simulation.devices = devices;
        self.session.simulation.fabrication = fabrication;
        self.notice = if cancelled {
            "Last build plan cancelled"
        } else {
            "No queued build plan"
        }
        .into();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldControl {
    Inspect,
    Excavate,
    Raise,
    Seal,
    Gate(u16),
    Recover,
    ToggleDevice,
}

pub fn field_control_action(control: FieldControl) -> UiAction {
    match control {
        FieldControl::Inspect => UiAction::Inspect,
        FieldControl::Excavate => UiAction::Terrain(crate::simulation::TerrainAction::Excavate),
        FieldControl::Raise => UiAction::Terrain(crate::simulation::TerrainAction::Raise),
        FieldControl::Seal => UiAction::Terrain(crate::simulation::TerrainAction::Seal),
        FieldControl::Gate(setting) => UiAction::SetGate(setting),
        FieldControl::Recover => UiAction::RecoverDeposit,
        FieldControl::ToggleDevice => UiAction::ToggleSelectedDevice,
    }
}

pub fn build_click_action(click: BuildClick) -> UiAction {
    match click {
        BuildClick::Palette(device) => UiAction::QueueDevice(device),
        BuildClick::Rotate => UiAction::RotatePlacement,
        BuildClick::Commit => UiAction::CommitPlan,
        BuildClick::Cancel => UiAction::CancelPlan,
        BuildClick::Time(time) => UiAction::SetTime(time),
    }
}

fn field_control_click() -> Option<FieldControl> {
    let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
    field_control_at(point.x, point.y)
}

pub fn field_control_at(x: f32, y: f32) -> Option<FieldControl> {
    if (1018.0..1248.0).contains(&x) && (532.0..560.0).contains(&y) {
        return Some(if x < 1126.0 {
            FieldControl::Inspect
        } else if x >= 1138.0 {
            FieldControl::Excavate
        } else {
            return None;
        });
    }
    if (1018.0..1248.0).contains(&x) && (566.0..594.0).contains(&y) {
        return Some(if x < 1126.0 {
            FieldControl::Raise
        } else if x >= 1138.0 {
            FieldControl::Seal
        } else {
            return None;
        });
    }
    if (1018.0..1248.0).contains(&x) && (600.0..628.0).contains(&y) {
        return Some(FieldControl::Gate(if x < 1074.0 {
            0
        } else if x < 1132.0 {
            2_500
        } else if x < 1190.0 {
            5_000
        } else {
            10_000
        }));
    }
    if (1018.0..1248.0).contains(&x) && (634.0..662.0).contains(&y) {
        return Some(if x < 1132.0 {
            FieldControl::Recover
        } else if x >= 1138.0 {
            FieldControl::ToggleDevice
        } else {
            return None;
        });
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildClick {
    Palette(DeviceId),
    Rotate,
    Commit,
    Cancel,
    Time(TimeControl),
}

fn mouse_build_click() -> Option<BuildClick> {
    let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
    build_click_at(point.x, point.y)
}

pub fn build_click_at(x: f32, y: f32) -> Option<BuildClick> {
    if (1018.0..1248.0).contains(&x) && (326.0..456.0).contains(&y) {
        let column = usize::from(x >= 1132.0);
        let row = ((y - 326.0) / 26.0) as usize;
        return DeviceId::ALL
            .get(row * 2 + column)
            .copied()
            .map(BuildClick::Palette);
    }
    if (1018.0..1248.0).contains(&x) && (468.0..494.0).contains(&y) {
        return if x < 1092.0 {
            Some(BuildClick::Rotate)
        } else if x < 1170.0 {
            Some(BuildClick::Commit)
        } else {
            Some(BuildClick::Cancel)
        };
    }
    if (1018.0..1250.0).contains(&x) && (496.0..522.0).contains(&y) {
        return Some(BuildClick::Time(if x < 1076.0 {
            TimeControl::Paused
        } else if x < 1136.0 {
            TimeControl::OneX
        } else if x < 1196.0 {
            TimeControl::TwoX
        } else {
            TimeControl::FourX
        }));
    }
    None
}
