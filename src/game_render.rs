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
        clear_background(Color::new(0.055, 0.052, 0.040, 1.0));
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
        let world = &self.session.simulation;
        draw_cube(
            vec3(world.width as f32 * 0.5, -0.32, world.height as f32 * 0.5),
            vec3(world.width as f32 + 18.0, 0.5, world.height as f32 + 18.0),
            None,
            Color::new(0.09, 0.085, 0.062, 1.0),
        );
        // The simulation is deliberately read directly here.  A former
        // presentation copy made mission briefings show a generic board until
        // the first tick, which was especially damaging to the diorama view.
        for y in 0..world.height {
            for x in 0..world.width {
                let pos = CellPos { x, y };
                let cell = &world.cells[world.index(pos).unwrap()];
                let h = cell.height_hu as f32 * 0.0005;
                let top = terrain_color(cell, x, y, self.session.mission.id);
                let base_tint = if self.overlay_mode == 0 && pos == selected {
                    blend(top, Color::new(0.95, 0.69, 0.16, 1.0), 0.48)
                } else {
                    top
                };
                let tint = if self.overlay_mode == 0 || self.overlay_mode == 2 {
                    base_tint
                } else {
                    overlay_tint(self.overlay_mode, cell.height_hu, cell)
                };
                draw_terrain_column(world, pos, h, tint, &self.terrain_texture);
                if self.overlay_mode == 0 && cell.surface_volume() == 0 {
                    draw_terrain_scatter(pos, h, self.session.mission.id);
                }
                let surface_depth = visual_fluid_depth(cell.surface_volume());
                if surface_depth > 0.0 {
                    let material = cell.surface.first().unwrap();
                    draw_fluid_surface(x, y, h, surface_depth, material.fluid, world.tick);
                    if self.overlay_mode == 2 {
                        draw_flow_arrow(world, pos, h + surface_depth + 0.065);
                    }
                }
                let steam_depth = cell.airborne_volume() as f32 * 0.00035;
                if steam_depth > 0.0 {
                    draw_steam(x, y, h, steam_depth, world.tick);
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
            draw_device_silhouette(
                device.device,
                center,
                width,
                height,
                device.active,
                device.rotation,
            );
            draw_blob_shadow(center, width as f32 * 0.9, height as f32 * 0.9);
            if device.anchor == selected {
                draw_cube_wires(
                    center,
                    vec3(width as f32 * 0.88, 0.95, height as f32 * 0.88),
                    Color::new(1.0, 0.75, 0.2, 0.95),
                );
            }
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
        for (index, definition) in self.session.simulation.definitions.iter().enumerate() {
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
                && pos == CellPos { x: 10, y: 6 };
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
        for event in &self.session.simulation.events {
            let crate::simulation::SimEvent::MaterialReacted {
                cell, volume_vu, ..
            } = event
            else {
                continue;
            };
            let index = self.session.simulation.index(*cell).unwrap();
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

fn draw_terrain_column(
    world: &crate::simulation::SimulationWorld,
    pos: CellPos,
    height: f32,
    top: Color,
    texture: &Texture2D,
) {
    let center = vec3(pos.x as f32 + 0.5, height * 0.5, pos.y as f32 + 0.5);
    // Dense wireframes made every map read as a debug grid.  A warm top and
    // shaded cliff block make elevation legible while retaining Macroquad's
    // inexpensive cube geometry.
    draw_cube(center, vec3(1.0, height.max(0.10), 1.0), Some(texture), top);
    let side = darken(top, 0.54);
    for (dx, dy, horizontal) in [
        (0i16, -1i16, true),
        (1, 0, false),
        (0, 1, true),
        (-1, 0, false),
    ] {
        let neighbor_height = world
            .index(CellPos {
                x: (pos.x as i16 + dx).max(0) as u16,
                y: (pos.y as i16 + dy).max(0) as u16,
            })
            .map(|i| world.cells[i].height_hu as f32 * 0.0005)
            .unwrap_or(0.0);
        if height > neighbor_height + 0.02 {
            let rise = height - neighbor_height;
            let c = if horizontal {
                vec3(
                    pos.x as f32 + 0.5,
                    neighbor_height + rise * 0.5,
                    pos.y as f32 + if dy < 0 { 0.01 } else { 0.99 },
                )
            } else {
                vec3(
                    pos.x as f32 + if dx < 0 { 0.01 } else { 0.99 },
                    neighbor_height + rise * 0.5,
                    pos.y as f32 + 0.5,
                )
            };
            let s = if horizontal {
                vec3(1.0, rise, 0.035)
            } else {
                vec3(0.035, rise, 1.0)
            };
            draw_cube(c, s, Some(texture), side);
        }
    }
}

fn draw_fluid_surface(x: u16, y: u16, ground: f32, depth: f32, fluid: FluidId, tick: u64) {
    let ripple = ((tick as f32 * 0.12 + x as f32 * 1.7 + y as f32).sin() + 1.0) * 0.012;
    let color = fluid_color(fluid);
    draw_cube(
        vec3(
            x as f32 + 0.5,
            ground + depth.max(0.035) * 0.5 + 0.018,
            y as f32 + 0.5,
        ),
        vec3(0.94, depth.max(0.035), 0.94),
        None,
        color,
    );
    draw_cube(
        vec3(x as f32 + 0.5, ground + depth + 0.038, y as f32 + 0.5),
        vec3(0.91, 0.012, 0.91),
        None,
        lighten(color, 0.22 + ripple),
    );
    let stripe = if (x + y) % 2 == 0 { 0.26 } else { -0.26 };
    draw_cube(
        vec3(
            x as f32 + 0.5 + stripe,
            ground + depth + 0.052 + ripple,
            y as f32 + 0.5,
        ),
        vec3(0.18, 0.014, 0.035),
        None,
        lighten(color, 0.38),
    );
}

fn visual_fluid_depth(volume_vu: u32) -> f32 {
    if volume_vu == 0 {
        0.0
    } else {
        0.055 + (volume_vu as f32 / 8_000.0).clamp(0.0, 1.0).sqrt() * 0.42
    }
}

fn draw_steam(x: u16, y: u16, ground: f32, depth: f32, tick: u64) {
    let drift = (tick as f32 * 0.08 + x as f32).sin() * 0.08;
    let color = Color::new(0.78, 0.88, 0.92, 0.28);
    for n in 0..3 {
        draw_sphere(
            vec3(
                x as f32 + 0.34 + n as f32 * 0.16 + drift,
                ground + 0.28 + depth * 0.35 + n as f32 * 0.12,
                y as f32 + 0.5,
            ),
            0.16 + n as f32 * 0.025,
            None,
            color,
        );
    }
}

fn draw_flow_arrow(world: &crate::simulation::SimulationWorld, pos: CellPos, y: f32) {
    let index = world.index(pos).unwrap();
    let head = world.cells[index].surface_head_hu();
    let candidates = [(1i16, 0i16), (-1, 0), (0, 1), (0, -1)];
    let Some((dx, dz, drop)) = candidates
        .into_iter()
        .filter_map(|(dx, dz)| {
            let target = CellPos {
                x: pos.x.checked_add_signed(dx)?,
                y: pos.y.checked_add_signed(dz)?,
            };
            let target = world.index(target)?;
            let difference = head - world.cells[target].surface_head_hu();
            (difference > 1).then_some((dx, dz, difference))
        })
        .max_by_key(|(_, _, difference)| *difference)
    else {
        return;
    };
    let length = (drop as f32 / 4_000.0).clamp(0.18, 0.48);
    let center = vec3(
        pos.x as f32 + 0.5 + dx as f32 * length * 0.16,
        y,
        pos.y as f32 + 0.5 + dz as f32 * length * 0.16,
    );
    let size = if dx != 0 {
        vec3(length, 0.018, 0.07)
    } else {
        vec3(0.07, 0.018, length)
    };
    draw_cube(center, size, None, Color::new(0.92, 0.97, 0.92, 0.92));
    let tip = center + vec3(dx as f32 * length * 0.55, 0.006, dz as f32 * length * 0.55);
    draw_sphere(tip, 0.07, None, Color::new(1.0, 0.84, 0.25, 0.96));
}

fn draw_blob_shadow(center: Vec3, width: f32, depth: f32) {
    draw_cube(
        vec3(center.x + 0.08, center.y - 0.34, center.z + 0.08),
        vec3(width, 0.018, depth),
        None,
        Color::new(0.03, 0.025, 0.02, 0.34),
    );
}

fn draw_terrain_scatter(pos: CellPos, height: f32, mission: crate::mission::MissionId) {
    let seed = (u32::from(pos.x) * 73 + u32::from(pos.y) * 151) % 97;
    let base = vec3(pos.x as f32 + 0.5, height + 0.045, pos.y as f32 + 0.5);
    if seed == 0 || seed == 3 {
        let offset = if seed == 0 {
            vec3(-0.22, 0.0, 0.17)
        } else {
            vec3(0.19, 0.0, -0.18)
        };
        draw_cube(
            base + offset + vec3(0.0, 0.07, 0.0),
            vec3(0.16, 0.13, 0.13),
            None,
            Color::new(0.22, 0.21, 0.18, 1.0),
        );
        draw_cube(
            base + offset + vec3(0.03, 0.15, -0.02),
            vec3(0.10, 0.10, 0.09),
            None,
            Color::new(0.29, 0.27, 0.22, 1.0),
        );
    }
    if matches!(
        mission,
        crate::mission::MissionId::L01FirstFlow | crate::mission::MissionId::L02HoldingLine
    ) && (seed == 11 || seed == 17)
    {
        let offset = if seed == 11 { -0.13 } else { 0.13 };
        for blade in 0..3 {
            draw_cube(
                base + vec3(
                    offset + blade as f32 * 0.05,
                    0.11,
                    -0.13 + blade as f32 * 0.04,
                ),
                vec3(0.018, 0.20 + blade as f32 * 0.035, 0.018),
                None,
                Color::new(0.33, 0.43, 0.23, 1.0),
            );
        }
    }
}

fn draw_device_silhouette(
    device: DeviceId,
    center: Vec3,
    width: u16,
    depth: u16,
    active: bool,
    rotation: u8,
) {
    let metal = Color::new(0.28, 0.31, 0.29, 1.0);
    let brass = Color::new(0.68, 0.47, 0.18, 1.0);
    let glow = if active {
        Color::new(0.12, 0.82, 0.78, 1.0)
    } else {
        Color::new(0.24, 0.42, 0.41, 1.0)
    };
    let base = vec3(width as f32 * 0.72, 0.18, depth as f32 * 0.72);
    draw_cube(center - vec3(0.0, 0.24, 0.0), base, None, metal);
    match device {
        DeviceId::Pipe => {
            let pipe_size = if rotation % 2 == 0 {
                vec3(0.88, 0.18, 0.24)
            } else {
                vec3(0.24, 0.18, 0.88)
            };
            draw_cube(center, pipe_size, None, brass);
            draw_cube(
                center + vec3(0.0, 0.12, 0.0),
                if rotation % 2 == 0 {
                    vec3(0.95, 0.04, 0.12)
                } else {
                    vec3(0.12, 0.04, 0.95)
                },
                None,
                glow,
            );
        }
        DeviceId::Floodgate => {
            draw_cube(
                center + vec3(0.0, 0.22, 0.0),
                vec3(0.8, 0.62, 0.15),
                None,
                metal,
            );
            draw_cube(
                center + vec3(0.0, 0.22, 0.10),
                vec3(0.48, 0.4, 0.035),
                None,
                glow,
            );
        }
        DeviceId::Pump => {
            draw_cube(
                center + vec3(0.0, 0.25, 0.0),
                vec3(0.48, 0.52, 0.48),
                None,
                brass,
            );
            draw_sphere(center + vec3(0.0, 0.56, 0.0), 0.18, None, glow);
        }
        DeviceId::Reservoir => {
            draw_cube(
                center + vec3(0.0, 0.45, 0.0),
                vec3(0.92, 0.92, 0.92),
                None,
                metal,
            );
            draw_cube(
                center + vec3(0.0, 0.48, 0.47),
                vec3(0.42, 0.46, 0.025),
                None,
                glow,
            );
        }
        DeviceId::FlowTurbine => {
            draw_cube(
                center + vec3(0.0, 0.28, 0.0),
                vec3(0.58, 0.6, 0.58),
                None,
                metal,
            );
            draw_sphere(center + vec3(0.0, 0.32, 0.31), 0.22, None, glow);
        }
        DeviceId::RuneRelay => {
            draw_cube(
                center + vec3(0.0, 0.36, 0.0),
                vec3(0.74, 0.72, 0.74),
                None,
                Color::new(0.32, 0.22, 0.42, 1.0),
            );
            draw_sphere(center + vec3(0.0, 0.82, 0.0), 0.18, None, glow);
        }
        DeviceId::Channel | DeviceId::Spillway => {
            draw_cube(
                center + vec3(0.0, 0.04, 0.0),
                vec3(0.84, 0.11, 0.84),
                None,
                brass,
            );
        }
        DeviceId::Sensor => {
            draw_cube(
                center + vec3(0.0, 0.42, 0.0),
                vec3(0.18, 0.82, 0.18),
                None,
                metal,
            );
            draw_sphere(center + vec3(0.0, 0.86, 0.0), 0.12, None, glow);
        }
        DeviceId::Filter => {
            draw_cube(
                center + vec3(0.0, 0.36, 0.0),
                vec3(0.56, 0.72, 0.56),
                None,
                metal,
            );
            draw_cube(
                center + vec3(0.0, 0.36, 0.30),
                vec3(0.24, 0.38, 0.03),
                None,
                glow,
            );
        }
    }
    let (dx, dz) = match rotation % 4 {
        0 => (0.46, 0.0),
        1 => (0.0, 0.46),
        2 => (-0.46, 0.0),
        _ => (0.0, -0.46),
    };
    draw_cube(
        center + vec3(dx, 0.12, dz),
        vec3(0.18, 0.12, 0.18),
        None,
        glow,
    );
}

fn terrain_color(
    cell: &crate::simulation::SimCell,
    x: u16,
    y: u16,
    mission: crate::mission::MissionId,
) -> Color {
    if cell.sealed {
        return Color::new(0.38, 0.37, 0.31, 1.0);
    }
    if cell.formed_rock_vu > 0 {
        return Color::new(0.18, 0.20, 0.21, 1.0);
    }
    let noise = ((x as f32 * 0.41 + y as f32 * 0.73).sin() + 1.0) * 0.035;
    match mission {
        crate::mission::MissionId::L03Firebreak => {
            Color::new(0.29 + noise, 0.20 + noise * 0.55, 0.14 + noise * 0.2, 1.0)
        }
        _ => Color::new(0.31 + noise, 0.32 + noise, 0.22 + noise * 0.45, 1.0),
    }
}
fn darken(c: Color, value: f32) -> Color {
    Color::new(c.r * value, c.g * value, c.b * value, c.a)
}
fn lighten(c: Color, value: f32) -> Color {
    Color::new(
        (c.r + value).min(1.0),
        (c.g + value).min(1.0),
        (c.b + value).min(1.0),
        c.a,
    )
}
fn blend(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        1.0,
    )
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
        FluidId::Water => Color::new(0.06, 0.38, 0.58, 0.82),
        FluidId::Lava => Color::new(0.96, 0.31, 0.045, 0.92),
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
