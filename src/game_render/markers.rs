//! Authored sources, protected anchors, drains, and reaction markers.

use super::Game;
use crate::state::CellPos;
use macroquad::prelude::*;

impl Game {
    pub(super) fn draw_authored_markers(&self) {
        self.draw_source_markers();
        self.draw_cell_markers();
        self.draw_reaction_markers();
    }

    fn draw_source_markers(&self) {
        for source in &self.session.simulation.sources {
            let Some(index) = self.session.simulation.index(source.position) else {
                continue;
            };
            let cell = &self.session.simulation.cells[index];
            let color = super::fluid_color(source.fluid);
            let base = cell.height_hu as f32 * 0.0005;
            draw_cube(
                vec3(
                    source.position.x as f32 + 0.5,
                    base + 0.22,
                    source.position.y as f32 + 0.5,
                ),
                vec3(0.56, 0.16, 0.56),
                None,
                Color::new(0.18, 0.17, 0.14, 1.0),
            );
            draw_sphere(
                vec3(
                    source.position.x as f32 + 0.5,
                    base + 0.25,
                    source.position.y as f32 + 0.5,
                ),
                0.18,
                None,
                color,
            );
        }
    }

    fn draw_cell_markers(&self) {
        for (index, definition) in self.session.simulation.definitions.iter().enumerate() {
            if definition.surface_drain_rate_vu > 0 {
                let x = (index % self.session.simulation.width as usize) as f32 + 0.5;
                let y = (index / self.session.simulation.width as usize) as f32 + 0.5;
                let height = self.session.simulation.cells[index].height_hu as f32 * 0.0005;
                draw_cube(
                    vec3(x, height + 0.015, y),
                    vec3(0.72, 0.03, 0.72),
                    None,
                    Color::new(0.025, 0.035, 0.045, 1.0),
                );
                draw_cube_wires(
                    vec3(x, height + 0.035, y),
                    vec3(0.76, 0.05, 0.76),
                    Color::new(0.32, 0.62, 0.68, 0.9),
                );
            }
            if !definition.protected {
                continue;
            }
            let x = (index % self.session.simulation.width as usize) as f32 + 0.5;
            let y = (index / self.session.simulation.width as usize) as f32 + 0.5;
            let height = self.session.simulation.cells[index].height_hu as f32 * 0.0005;
            let pos = CellPos {
                x: x.floor() as u16,
                y: y.floor() as u16,
            };
            let is_beacon = self.session.mission.id == crate::mission::MissionId::L01FirstFlow
                && pos == CellPos { x: 24, y: 8 };
            if is_beacon {
                draw_cube(
                    vec3(x, height + 0.35, y),
                    vec3(0.22, 0.7, 0.22),
                    None,
                    Color::new(0.42, 0.28, 0.54, 1.0),
                );
                draw_sphere(
                    vec3(x, height + 0.76, y),
                    0.15,
                    None,
                    Color::new(0.76, 0.55, 0.88, 1.0),
                );
            } else if (u32::from(pos.x) * 3 + u32::from(pos.y) * 5) % 11 == 0 {
                draw_cube(
                    vec3(x, height + 0.025, y),
                    vec3(0.34, 0.045, 0.34),
                    None,
                    Color::new(0.52, 0.42, 0.50, 0.72),
                );
            }
        }
    }

    fn draw_reaction_markers(&self) {
        for event in &self.session.simulation.events {
            let crate::simulation::SimEvent::MaterialReacted {
                cell, volume_vu, ..
            } = event
            else {
                continue;
            };
            let Some(index) = self.session.simulation.index(*cell) else {
                continue;
            };
            let base = self.session.simulation.cells[index].height_hu as f32 * 0.0005;
            let scale = (*volume_vu as f32 / 250.0).clamp(0.35, 1.0);
            draw_sphere(
                vec3(
                    cell.x as f32 + 0.5,
                    base + 0.35 + scale * 0.22,
                    cell.y as f32 + 0.5,
                ),
                0.16 + scale * 0.12,
                None,
                Color::new(0.94, 0.78, 0.40, 0.78),
            );
            draw_cube(
                vec3(cell.x as f32 + 0.5, base + 0.07, cell.y as f32 + 0.5),
                vec3(0.36, 0.14, 0.36),
                None,
                Color::new(0.20, 0.18, 0.16, 1.0),
            );
        }
    }
}
