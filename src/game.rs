//! Foundation orchestration: input, fixed ticks, orthographic world, HUD.

use crate::{data::GameData, simulation::{FluidId, TerrainAction}, state::{save_session, load_session, CellPos, GameSession, TimeControl}, verification::FluidsLab};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, end_virtual_ui_frame};
use crate::ui::{self, UiContext};

pub struct Game { pub data: GameData, pub session: GameSession, assets: AssetManager, camera: FoundationCamera, lab: FluidsLab, notice: String }

#[derive(Debug, Clone, Copy)]
struct FoundationCamera { target: Vec2, yaw: u8, zoom: f32 }

impl FoundationCamera {
    fn new(width: usize, height: usize) -> Self { Self { target: vec2(width as f32 / 2.0, height as f32 / 2.0), yaw: 0, zoom: 16.0 } }
    fn update(&mut self, dt: f32, width: usize, height: usize) {
        let speed = dt * self.zoom * 0.8;
        if is_key_down(KeyCode::A) { self.target.x -= speed; }
        if is_key_down(KeyCode::D) { self.target.x += speed; }
        if is_key_down(KeyCode::W) { self.target.y -= speed; }
        if is_key_down(KeyCode::S) { self.target.y += speed; }
        self.target.x = self.target.x.clamp(0.0, width as f32); self.target.y = self.target.y.clamp(0.0, height as f32);
        if is_key_pressed(KeyCode::Q) { self.yaw = (self.yaw + 3) % 4; }
        if is_key_pressed(KeyCode::E) { self.yaw = (self.yaw + 1) % 4; }
        if is_key_pressed(KeyCode::Minus) { self.zoom = (self.zoom + 4.0).min(54.0); }
        if is_key_pressed(KeyCode::Equal) { self.zoom = (self.zoom - 4.0).max(12.0); }
    }
    fn camera3d(&self) -> Camera3D {
        let angle = self.yaw as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        let distance = self.zoom * 1.65;
        let target = vec3(self.target.x, 0.0, self.target.y);
        Camera3D { position: target + vec3(angle.cos() * distance, distance * 0.82, angle.sin() * distance), target, up: vec3(0.0, 1.0, 0.0), projection: Projection::Orthographics, fovy: self.zoom, aspect: Some(screen_width() / screen_height()), ..Default::default() }
    }
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().expect("embedded foundation data must be valid");
        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(8, 8, Color::new(0.7, 0.25, 0.35, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let _ = assets.load_asset_pack("assets.zip").await;
        let _ = assets.load_texture_configs(&data.texture_manifest).await;
        let session = GameSession::new(&data.config);
        let camera = FoundationCamera::new(data.config.world_width, data.config.world_height);
        Self { data, session, assets, camera, lab: FluidsLab::new(), notice: "Foundation online — simulation paused".into() }
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt, self.data.config.world_width, self.data.config.world_height);
        if is_key_pressed(KeyCode::Space) { self.session.time_control = match self.session.time_control { TimeControl::Paused => TimeControl::OneX, TimeControl::OneX => TimeControl::Paused, _ => TimeControl::Paused }; }
        if is_key_pressed(KeyCode::Key1) { self.session.time_control = TimeControl::OneX; }
        if is_key_pressed(KeyCode::Key2) { self.session.time_control = TimeControl::TwoX; }
        if is_key_pressed(KeyCode::Key4) { self.session.time_control = TimeControl::FourX; }
        if is_key_pressed(KeyCode::Up) { self.session.move_selected(0, -1); }
        if is_key_pressed(KeyCode::Down) { self.session.move_selected(0, 1); }
        if is_key_pressed(KeyCode::Left) { self.session.move_selected(-1, 0); }
        if is_key_pressed(KeyCode::Right) { self.session.move_selected(1, 0); }
        if is_key_pressed(KeyCode::X) { self.apply_terrain(TerrainAction::Excavate); }
        if is_key_pressed(KeyCode::R) { self.apply_terrain(TerrainAction::Raise); }
        if is_key_pressed(KeyCode::T) { self.apply_terrain(TerrainAction::Seal); }
        if is_key_pressed(KeyCode::I) { self.session.simulation.inject(self.session.selected, FluidId::Water, 500); }
        if is_key_pressed(KeyCode::L) { self.session.simulation.inject(self.session.selected, FluidId::Lava, 500); }
        if is_key_pressed(KeyCode::G) { self.session.simulation.inject(self.session.selected, FluidId::ToxicSlurry, 500); }
        if is_key_pressed(KeyCode::F1) { let report = self.lab.automatic_scenario(); self.notice = format!("lab_fluids_all {} tick {} hash {:016X}", if report.passed { "PASS" } else { "FAIL" }, report.tick, report.state_hash); }
        if is_key_pressed(KeyCode::F5) { self.notice = save_session(&self.session, &self.data.config).map(|_| "Checkpoint saved".into()).unwrap_or_else(|e| e); }
        if is_key_pressed(KeyCode::F9) { match load_session(&self.data.config) { Ok(s) => { self.session = s; self.notice = "Checkpoint loaded".into(); }, Err(e) => self.notice = e } }
        let ticks = self.session.update(dt); if ticks > 0 { self.notice = format!("Simulation advanced {ticks} tick(s)"); }
    }

    fn apply_terrain(&mut self, action: TerrainAction) {
        self.notice = self.session.simulation.terrain_edit(self.session.selected, action).map(|_| "Terrain edit committed".into()).unwrap_or_else(|error| format!("Terrain edit rejected: {error:?}"));
        if let Some(index) = self.session.simulation.index(self.session.selected) { self.session.world.cells[index].height_hu = self.session.simulation.cells[index].height_hu; self.session.world.cells[index].sealed = self.session.simulation.cells[index].sealed; }
    }

    pub fn draw(&mut self) {
        clear_background(Color::new(0.035, 0.045, 0.065, 1.0));
        set_camera(&self.camera.camera3d());
        self.draw_world();
        set_default_camera();
        begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        ui::draw_hud(UiContext { data: &self.data, session: &self.session, camera_yaw: self.camera.yaw, camera_zoom: self.camera.zoom, notice: &self.notice, loaded_assets: self.assets.len() });
        end_virtual_ui_frame();
    }

    fn draw_world(&self) {
        let selected = self.session.selected;
        for y in 0..self.session.world.height { for x in 0..self.session.world.width {
            let pos = CellPos { x, y }; let cell = &self.session.world.cells[self.session.world.index(pos).unwrap()];
            let h = cell.height_hu as f32 * 0.0005; let center = vec3(x as f32 + 0.5, h * 0.5, y as f32 + 0.5); let size = vec3(0.96, h.max(0.12), 0.96);
            let tint = if pos == selected { Color::new(0.82, 0.66, 0.24, 1.0) } else if cell.sealed { Color::new(0.25, 0.29, 0.35, 1.0) } else { Color::new(0.32 + x as f32 * 0.005, 0.24 + y as f32 * 0.004, 0.20, 1.0) };
            draw_cube(center, size, None, tint); draw_cube_wires(center, size, Color::new(0.08, 0.09, 0.12, 0.55));
        }}
        draw_grid_lines();
    }
}

fn draw_grid_lines() { /* Depth-tested cube silhouettes provide the stepped grid in R0. */ }
