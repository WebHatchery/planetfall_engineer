//! Player-facing placement preview derived from authoritative footprint rules.

use crate::game::Game;
use crate::state::CellPos;
use macroquad::prelude::*;

impl Game {
    pub(crate) fn draw_placement_ghost(&self) {
        let anchor = self.session.selected;
        let (width, height) = self.placement_device.footprint();
        let valid = anchor.x + width <= self.session.world.width
            && anchor.y + height <= self.session.world.height
            && !(0..height).any(|dy| {
                (0..width).any(|dx| {
                    let pos = CellPos {
                        x: anchor.x + dx,
                        y: anchor.y + dy,
                    };
                    self.session
                        .simulation
                        .devices
                        .devices
                        .iter()
                        .any(|placed| {
                            let (placed_width, placed_height) = placed.device.footprint();
                            pos.x >= placed.anchor.x
                                && pos.x < placed.anchor.x + placed_width
                                && pos.y >= placed.anchor.y
                                && pos.y < placed.anchor.y + placed_height
                        })
                })
            });
        if !valid {
            return;
        }
        let cell = &self.session.simulation.cells[self.session.simulation.index(anchor).unwrap()];
        let base = cell.height_hu as f32 * 0.0005;
        let center = vec3(
            anchor.x as f32 + width as f32 * 0.5,
            base + 0.38,
            anchor.y as f32 + height as f32 * 0.5,
        );
        draw_cube(
            center,
            vec3(width as f32 * 0.68, 0.18, height as f32 * 0.68),
            None,
            Color::new(0.35, 0.82, 0.96, 0.38),
        );
        draw_cube_wires(
            center,
            vec3(width as f32 * 0.76, 0.22, height as f32 * 0.76),
            Color::new(0.55, 0.92, 1.0, 0.95),
        );
    }
}
