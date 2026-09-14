//! Mouse and keyboard build-palette actions.

use crate::game::Game;
use crate::state::TimeControl;
use crate::{devices::DeviceId, ui, ui_action::UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::{input::hit_test, ui::virtual_mouse_position};

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
    hit_test(
        [
            (Rect::new(1018.0, 532.0, 108.0, 28.0), FieldControl::Inspect),
            (
                Rect::new(1138.0, 532.0, 108.0, 28.0),
                FieldControl::Excavate,
            ),
            (Rect::new(1018.0, 566.0, 108.0, 28.0), FieldControl::Raise),
            (Rect::new(1138.0, 566.0, 108.0, 28.0), FieldControl::Seal),
            (Rect::new(1018.0, 600.0, 54.0, 28.0), FieldControl::Gate(0)),
            (
                Rect::new(1076.0, 600.0, 54.0, 28.0),
                FieldControl::Gate(2_500),
            ),
            (
                Rect::new(1134.0, 600.0, 54.0, 28.0),
                FieldControl::Gate(5_000),
            ),
            (
                Rect::new(1192.0, 600.0, 54.0, 28.0),
                FieldControl::Gate(10_000),
            ),
            (Rect::new(1018.0, 634.0, 108.0, 28.0), FieldControl::Recover),
            (
                Rect::new(1138.0, 634.0, 108.0, 28.0),
                FieldControl::ToggleDevice,
            ),
        ]
        .into_iter()
        .map(|(rect, value)| macroquad_toolkit::input::HitTarget::new(rect, value)),
        vec2(x, y),
    )
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
    let palette = DeviceId::ALL
        .into_iter()
        .enumerate()
        .map(|(index, device)| {
            let column = index % 2;
            let row = index / 2;
            macroquad_toolkit::input::HitTarget::new(
                Rect::new(
                    1018.0 + column as f32 * 114.0,
                    344.0 + row as f32 * 24.0,
                    108.0,
                    22.0,
                ),
                BuildClick::Palette(device),
            )
        });
    let build = [
        (Rect::new(1018.0, 468.0, 72.0, 22.0), BuildClick::Rotate),
        (Rect::new(1094.0, 468.0, 72.0, 22.0), BuildClick::Commit),
        (Rect::new(1172.0, 468.0, 72.0, 22.0), BuildClick::Cancel),
    ]
    .into_iter()
    .map(|(rect, value)| macroquad_toolkit::input::HitTarget::new(rect, value));
    let time = [
        (Rect::new(1018.0, 496.0, 52.0, 22.0), TimeControl::Paused),
        (Rect::new(1078.0, 496.0, 52.0, 22.0), TimeControl::OneX),
        (Rect::new(1138.0, 496.0, 52.0, 22.0), TimeControl::TwoX),
        (Rect::new(1198.0, 496.0, 52.0, 22.0), TimeControl::FourX),
    ]
    .into_iter()
    .map(|(rect, value)| macroquad_toolkit::input::HitTarget::new(rect, BuildClick::Time(value)));
    hit_test(palette.chain(build).chain(time), vec2(x, y))
}
