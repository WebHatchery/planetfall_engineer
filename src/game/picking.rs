//! Screen-to-cell picking for the active 3D survey board.

use super::Game;
use crate::{mission::CommandKind, state::CellPos, ui_action::UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::render3d::picking::{screen_ray, Aabb3};

impl Game {
    pub(crate) fn select_from_pointer(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        if !(82.0..=604.0).contains(&mouse_y) || mouse_x > 1_000.0 {
            return;
        }
        let ray = screen_ray(&self.camera.camera3d(), vec2(mouse_x, mouse_y), None);
        let mut closest: Option<(CellPos, f32)> = None;
        for y in 0..self.session.simulation.height {
            for x in 0..self.session.simulation.width {
                let pos = CellPos { x, y };
                let Some(index) = self.session.simulation.index(pos) else {
                    continue;
                };
                let cell = &self.session.simulation.cells[index];
                let height = (cell.height_hu as f32 * 0.0005).max(0.12);
                if let Some(distance) = Aabb3::from_center_size(
                    vec3(x as f32 + 0.5, height * 0.5, y as f32 + 0.5),
                    vec3(0.96, height, 0.96),
                )
                .intersect(ray)
                {
                    if closest.is_none_or(|(_, current)| distance < current) {
                        closest = Some((pos, distance));
                    }
                }
            }
        }
        for device in &self.session.simulation.devices.devices {
            let (width, height) = device.device.footprint();
            let Some(index) = self.session.simulation.index(device.anchor) else {
                continue;
            };
            let cell = &self.session.simulation.cells[index];
            let base = cell.height_hu as f32 * 0.0005;
            if let Some(distance) = Aabb3::from_center_size(
                vec3(
                    device.anchor.x as f32 + width as f32 * 0.5,
                    base + 0.35,
                    device.anchor.y as f32 + height as f32 * 0.5,
                ),
                vec3(width as f32 * 0.72, 0.7, height as f32 * 0.72),
            )
            .intersect(ray)
            {
                if closest.is_none_or(|(_, current)| distance < current) {
                    closest = Some((device.anchor, distance));
                }
            }
        }
        if let Some((selected, _)) = closest {
            if selected == self.session.selected
                && self
                    .session
                    .mission
                    .tutorial
                    .as_ref()
                    .is_some_and(|tutorial| {
                        tutorial.current_step_id == "tutorial_l01_inspect_grade"
                    })
            {
                let _ = self.admit(CommandKind::Inspect);
                return;
            }
            self.dispatch_action(UiAction::Select(selected));
        }
    }
}
