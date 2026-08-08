//! Mouse and keyboard build-palette actions.

use crate::game::Game;
use crate::state::TimeControl;
use crate::{devices::DeviceId, ui};
use macroquad::prelude::*;
use macroquad_toolkit::ui::virtual_mouse_position;

impl Game {
    pub(crate) fn handle_palette_click(&mut self) -> bool {
        if let Some(BuildClick::Palette(device)) = mouse_build_click() {
            self.queue_device(device);
            return true;
        }
        false
    }

    pub(crate) fn handle_build_action_click(&mut self) -> bool {
        match mouse_build_click() {
            Some(BuildClick::Rotate) => {
                self.placement_rotation = (self.placement_rotation + 1) % 4;
                self.notice = format!("Placement rotation {}°", self.placement_rotation * 90);
                true
            }
            Some(BuildClick::Commit) => {
                self.commit_build_plan();
                true
            }
            Some(BuildClick::Cancel) => {
                self.cancel_build_plan();
                true
            }
            _ => false,
        }
    }

    pub(crate) fn handle_time_click(&mut self) -> bool {
        if let Some(BuildClick::Time(time)) = mouse_build_click() {
            self.set_time(time);
            true
        } else {
            false
        }
    }

    pub(crate) fn handle_field_control_click(&mut self) -> bool {
        match field_control_click() {
            Some(FieldControl::Inspect) => {
                if self.admit(crate::mission::CommandKind::Inspect) {
                    self.notice = format!(
                        "Inspecting cell {}, {}",
                        self.session.selected.x, self.session.selected.y
                    );
                }
                true
            }
            Some(FieldControl::Excavate) => {
                self.apply_terrain(crate::simulation::TerrainAction::Excavate);
                true
            }
            Some(FieldControl::Raise) => {
                self.apply_terrain(crate::simulation::TerrainAction::Raise);
                true
            }
            Some(FieldControl::Seal) => {
                self.apply_terrain(crate::simulation::TerrainAction::Seal);
                true
            }
            Some(FieldControl::Gate(setting)) => {
                self.set_gate(setting);
                true
            }
            None => false,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldControl {
    Inspect,
    Excavate,
    Raise,
    Seal,
    Gate(u16),
}

fn field_control_click() -> Option<FieldControl> {
    let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
    field_control_at(point.x, point.y)
}

fn field_control_at(x: f32, y: f32) -> Option<FieldControl> {
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
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildClick {
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

fn build_click_at(x: f32, y: f32) -> Option<BuildClick> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_visible_build_control_maps_to_its_action() {
        assert_eq!(
            build_click_at(1040.0, 337.0),
            Some(BuildClick::Palette(DeviceId::Channel))
        );
        assert_eq!(
            build_click_at(1200.0, 441.0),
            Some(BuildClick::Palette(DeviceId::RuneRelay))
        );
        assert_eq!(build_click_at(1040.0, 480.0), Some(BuildClick::Rotate));
        assert_eq!(build_click_at(1120.0, 480.0), Some(BuildClick::Commit));
        assert_eq!(build_click_at(1200.0, 480.0), Some(BuildClick::Cancel));
    }

    #[test]
    fn every_time_button_maps_to_the_expected_speed() {
        assert_eq!(
            build_click_at(1040.0, 507.0),
            Some(BuildClick::Time(TimeControl::Paused))
        );
        assert_eq!(
            build_click_at(1100.0, 507.0),
            Some(BuildClick::Time(TimeControl::OneX))
        );
        assert_eq!(
            build_click_at(1160.0, 507.0),
            Some(BuildClick::Time(TimeControl::TwoX))
        );
        assert_eq!(
            build_click_at(1220.0, 507.0),
            Some(BuildClick::Time(TimeControl::FourX))
        );
        assert_eq!(build_click_at(900.0, 507.0), None);
    }

    #[test]
    fn every_field_action_has_a_visible_touch_target() {
        assert_eq!(field_control_at(1040.0, 546.0), Some(FieldControl::Inspect));
        assert_eq!(
            field_control_at(1200.0, 546.0),
            Some(FieldControl::Excavate)
        );
        assert_eq!(field_control_at(1040.0, 580.0), Some(FieldControl::Raise));
        assert_eq!(field_control_at(1200.0, 580.0), Some(FieldControl::Seal));
        assert_eq!(field_control_at(1040.0, 614.0), Some(FieldControl::Gate(0)));
        assert_eq!(
            field_control_at(1100.0, 614.0),
            Some(FieldControl::Gate(2_500))
        );
        assert_eq!(
            field_control_at(1160.0, 614.0),
            Some(FieldControl::Gate(5_000))
        );
        assert_eq!(
            field_control_at(1220.0, 614.0),
            Some(FieldControl::Gate(10_000))
        );
    }
}
