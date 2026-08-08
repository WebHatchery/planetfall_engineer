//! Production 3D drawing and HUD restoration.

use crate::{
    devices::DeviceId,
    game::{FrontendMode, Game, VerificationMode},
    simulation::FluidId,
    state::CellPos,
    ui::{self, UiContext},
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, end_virtual_ui_frame};

impl Game {
    pub fn draw(&mut self) {
        clear_background(Color::new(0.035, 0.045, 0.065, 1.0));
        if self.frontend_mode != FrontendMode::Playing {
            begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
            self.draw_frontend();
            end_virtual_ui_frame();
            return;
        }
        set_camera(&self.camera.camera3d());
        self.draw_world();
        set_default_camera();
        let (placement_valid, placement_reason) = self.placement_preview();
        begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        ui::draw_hud(UiContext {
            session: &self.session,
            camera_yaw: self.camera.yaw,
            camera_zoom: self.camera.zoom,
            notice: &self.notice,
            loaded_assets: self.assets.len(),
            verification_label: self.verification_mode.map(verification_label),
            pause_menu: self.pause_menu,
            placement_device: self.placement_device,
            placement_rotation: self.placement_rotation,
            placement_valid,
            placement_reason,
            overlay_mode: self.overlay_mode,
        });
        end_virtual_ui_frame();
    }

    fn draw_world(&self) {
        let selected = self.session.selected;
        for y in 0..self.session.world.height {
            for x in 0..self.session.world.width {
                let pos = CellPos { x, y };
                let cell = &self.session.world.cells[self.session.world.index(pos).unwrap()];
                let h = cell.height_hu as f32 * 0.0005;
                let center = vec3(x as f32 + 0.5, h * 0.5, y as f32 + 0.5);
                let size = vec3(0.96, h.max(0.12), 0.96);
                let base_tint = if self.overlay_mode == 0 && pos == selected {
                    Color::new(0.82, 0.66, 0.24, 1.0)
                } else if cell.sealed {
                    Color::new(0.25, 0.29, 0.35, 1.0)
                } else {
                    Color::new(0.32 + x as f32 * 0.005, 0.24 + y as f32 * 0.004, 0.20, 1.0)
                };
                let sim_cell =
                    &self.session.simulation.cells[self.session.simulation.index(pos).unwrap()];
                let tint = if self.overlay_mode == 0 {
                    base_tint
                } else {
                    overlay_tint(self.overlay_mode, cell.height_hu, sim_cell)
                };
                draw_cube(center, size, None, tint);
                draw_cube_wires(center, size, Color::new(0.08, 0.09, 0.12, 0.55));
                let surface_depth = sim_cell.surface_volume() as f32 * 0.0005;
                if surface_depth > 0.0 {
                    draw_cube(
                        vec3(
                            x as f32 + 0.5,
                            h + surface_depth * 0.5 + 0.02,
                            y as f32 + 0.5,
                        ),
                        vec3(0.88, surface_depth.max(0.04), 0.88),
                        None,
                        sim_cell
                            .surface
                            .first()
                            .map(|material| fluid_color(material.fluid))
                            .unwrap_or(WHITE),
                    );
                }
                let steam_depth = sim_cell.airborne_volume() as f32 * 0.00035;
                if steam_depth > 0.0 {
                    draw_cube(
                        vec3(x as f32 + 0.5, h + 0.35 + steam_depth * 0.5, y as f32 + 0.5),
                        vec3(0.7, steam_depth.max(0.08), 0.7),
                        None,
                        Color::new(0.76, 0.86, 0.92, 0.38),
                    );
                }
            }
        }
        for device in &self.session.simulation.devices.devices {
            let (width, height) = device.device.footprint();
            let anchor = &self.session.simulation.cells
                [self.session.simulation.index(device.anchor).unwrap()];
            let center = vec3(
                device.anchor.x as f32 + width as f32 * 0.5,
                anchor.height_hu as f32 * 0.0005 + 0.35,
                device.anchor.y as f32 + height as f32 * 0.5,
            );
            let color = if device.active {
                Color::new(0.35, 0.92, 0.72, 1.0)
            } else {
                Color::new(0.72, 0.52, 0.25, 1.0)
            };
            draw_cube(
                center,
                vec3(width as f32 * 0.72, 0.7, height as f32 * 0.72),
                None,
                color,
            );
            draw_cube_wires(
                center,
                vec3(width as f32 * 0.78, 0.74, height as f32 * 0.78),
                if device.anchor == selected {
                    WHITE
                } else {
                    Color::new(0.08, 0.09, 0.12, 0.8)
                },
            );
        }
        self.draw_authored_markers();
        self.draw_placement_ghost();
    }

    fn draw_authored_markers(&self) {
        for source in &self.session.simulation.sources {
            let cell = &self.session.simulation.cells
                [self.session.simulation.index(source.position).unwrap()];
            let color = fluid_color(source.fluid);
            let base = cell.height_hu as f32 * 0.0005;
            draw_cube(
                vec3(
                    source.position.x as f32 + 0.5,
                    base + 0.22,
                    source.position.y as f32 + 0.5,
                ),
                vec3(0.42, 0.44, 0.42),
                None,
                color,
            );
            draw_cube_wires(
                vec3(
                    source.position.x as f32 + 0.5,
                    base + 0.22,
                    source.position.y as f32 + 0.5,
                ),
                vec3(0.5, 0.5, 0.5),
                WHITE,
            );
        }
        for (index, definition) in self.session.simulation.definitions.iter().enumerate() {
            if !definition.protected {
                continue;
            }
            let x = (index % self.session.simulation.width as usize) as f32 + 0.5;
            let y = (index / self.session.simulation.width as usize) as f32 + 0.5;
            let height = self.session.simulation.cells[index].height_hu as f32 * 0.0005;
            draw_cube(
                vec3(x, height + 0.28, y),
                vec3(0.34, 0.56, 0.34),
                None,
                Color::new(0.74, 0.46, 0.86, 1.0),
            );
            draw_cube_wires(
                vec3(x, height + 0.28, y),
                vec3(0.42, 0.64, 0.42),
                Color::new(0.96, 0.86, 0.98, 1.0),
            );
        }
    }
}

fn verification_label(mode: VerificationMode) -> &'static str {
    match mode {
        VerificationMode::Lab => "LAB_FLUIDS_ALL",
        VerificationMode::Showcase(device) => match device {
            DeviceId::Channel => "DEVICE_CHANNEL",
            DeviceId::Pipe => "DEVICE_PIPE",
            DeviceId::Pump => "DEVICE_PUMP",
            DeviceId::Floodgate => "DEVICE_FLOODGATE",
            DeviceId::Reservoir => "DEVICE_RESERVOIR",
            DeviceId::Spillway => "DEVICE_SPILLWAY",
            DeviceId::FlowTurbine => "DEVICE_FLOW_TURBINE",
            DeviceId::Sensor => "DEVICE_SENSOR",
            DeviceId::Filter => "DEVICE_FILTER",
            DeviceId::RuneRelay => "DEVICE_RUNE_RELAY",
        },
    }
}

fn fluid_color(fluid: FluidId) -> Color {
    match fluid {
        FluidId::Water => Color::new(0.12, 0.48, 0.9, 0.78),
        FluidId::Lava => Color::new(0.95, 0.22, 0.06, 0.9),
        FluidId::ToxicSlurry => Color::new(0.62, 0.72, 0.16, 0.86),
        FluidId::Steam => Color::new(0.76, 0.86, 0.92, 0.38),
    }
}

fn overlay_tint(mode: u8, height_hu: i16, cell: &crate::simulation::SimCell) -> Color {
    match mode {
        1 => {
            let value = (height_hu as f32 / 8_000.0).clamp(0.0, 1.0);
            Color::new(0.12 + value * 0.72, 0.24 + (1.0 - value) * 0.44, 0.34, 1.0)
        }
        2 => {
            let value = (cell.surface_volume() as f32 / 8_000.0).clamp(0.0, 1.0);
            Color::new(0.12, 0.3 + value * 0.6, 0.65 + value * 0.25, 1.0)
        }
        3 => {
            let temperature = cell
                .surface
                .first()
                .map(|material| material.temperature_dk)
                .unwrap_or(2_930);
            let value = ((temperature - 2_930) as f32 / 10_000.0).clamp(0.0, 1.0);
            Color::new(0.18 + value * 0.78, 0.28 + (1.0 - value) * 0.32, 0.22, 1.0)
        }
        4 => {
            let value = cell.ground_contamination_bp as f32 / 10_000.0;
            Color::new(0.22 + value * 0.7, 0.24 + (1.0 - value) * 0.4, 0.12, 1.0)
        }
        _ => Color::new(0.32, 0.24, 0.2, 1.0),
    }
}
