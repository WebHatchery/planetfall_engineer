//! Foundation orchestration: input, fixed ticks, orthographic world, HUD.

use crate::{campaign::{load_campaign, seed_reference_materials}, data::GameData, devices::{run_all_showcases, DeviceId}, mission::{campaign_summary, CommandKind, MissionId, MissionPhase}, replay::run_all_references, simulation::{FluidId, TerrainAction}, state::{save_session, load_session, CellPos, GameSession, TimeControl, WorldState}, verification::FluidsLab};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, end_virtual_ui_frame};
use macroquad_toolkit::render3d::picking::{screen_ray, Aabb3};
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
        let viewport_scale = (screen_width() / ui::LOGICAL_WIDTH).min(screen_height() / ui::LOGICAL_HEIGHT).max(0.5);
        Camera3D { position: target + vec3(angle.cos() * distance, distance * 0.82, angle.sin() * distance), target, up: vec3(0.0, 1.0, 0.0), projection: Projection::Orthographics, fovy: self.zoom / viewport_scale, aspect: Some(screen_width() / screen_height()), ..Default::default() }
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
        let mut session = GameSession::new(&data.config);
        let campaign = load_campaign(MissionId::L01FirstFlow);
        session.simulation = campaign.world;
        session.world = WorldState::new(32, 20);
        let camera = FoundationCamera::new(data.config.world_width, data.config.world_height);
        let (width, height) = MissionId::L01FirstFlow.map_size();
        Self { data, session, assets, camera, lab: FluidsLab::new(), notice: format!("First Flow briefing active — {width}×{height} — budget {} — reference {}–{} ticks", campaign.budget, campaign.reference_tick_range.0, campaign.reference_tick_range.1) }
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt, self.data.config.world_width, self.data.config.world_height);
        if is_mouse_button_pressed(MouseButton::Left) { self.select_from_pointer(); }
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
        if is_key_pressed(KeyCode::F2) { self.notice = run_all_showcases(); }
        if is_key_pressed(KeyCode::F3) { self.notice = campaign_summary(); }
        if is_key_pressed(KeyCode::F4) { self.session.mission.skip_tutorial(); let complete = self.session.mission.tutorial.as_ref().is_some_and(|tutorial| tutorial.is_complete()); self.notice = if complete { "Tutorial skipped — L01 build kit unlocked" } else { "Tutorial skip unavailable" }.into(); }
        if is_key_pressed(KeyCode::F6) { self.session.mission.checkpoint(); self.notice = format!("Mission checkpoint recorded at tick {}", self.session.mission.checkpoint_tick); }
        if is_key_pressed(KeyCode::F7) { self.session.mission.fail("manual failure-path check"); self.notice = "Mission failed — reset to checkpoint".into(); }
        if is_key_pressed(KeyCode::F8) { let admission = self.session.mission.admit(CommandKind::DismissPrompt); self.notice = format!("Tutorial command: {admission:?}"); }
        if is_key_pressed(KeyCode::F10) { let mut campaign = load_campaign(self.session.mission.id); seed_reference_materials(&mut campaign); self.session.simulation = campaign.world; self.notice = "Reference material fixture loaded".into(); }
        if is_key_pressed(KeyCode::F11) { self.notice = run_all_references(); }
        if is_key_pressed(KeyCode::C) { if let Some(entity_id) = self.session.simulation.devices.devices.iter().find(|device| device.anchor == self.session.selected).map(|device| device.entity_id) { self.session.simulation.devices.remove(entity_id); self.notice = "Device removed and budget released".into(); } }
        if is_key_pressed(KeyCode::B) { self.queue_device(DeviceId::Channel); }
        if is_key_pressed(KeyCode::P) { self.queue_device(DeviceId::Pipe); }
        if is_key_pressed(KeyCode::O) { self.queue_device(DeviceId::Pump); }
        if is_key_pressed(KeyCode::Enter) { self.commit_build_plan(); }
        if is_key_pressed(KeyCode::Backspace) { self.cancel_build_plan(); }
        if is_key_pressed(KeyCode::F5) { self.notice = save_session(&self.session, &self.data.config).map(|_| "Checkpoint saved".into()).unwrap_or_else(|e| e); }
        if is_key_pressed(KeyCode::F9) { match load_session(&self.data.config) { Ok(s) => { self.session = s; self.notice = "Checkpoint loaded".into(); }, Err(e) => self.notice = e } }
        let ticks = self.session.update(dt);
        if ticks > 0 { self.session.mission.on_tick(&self.session.simulation); if self.session.mission.phase == MissionPhase::Success { self.session.campaign.record_success(self.session.mission.id, self.session.mission.tick); } self.notice = format!("Simulation advanced {ticks} tick(s)"); }
    }

    fn apply_terrain(&mut self, action: TerrainAction) {
        self.notice = self.session.simulation.terrain_edit(self.session.selected, action).map(|_| "Terrain edit committed".into()).unwrap_or_else(|error| format!("Terrain edit rejected: {error:?}"));
        if let Some(index) = self.session.simulation.index(self.session.selected) { self.session.world.cells[index].height_hu = self.session.simulation.cells[index].height_hu; self.session.world.cells[index].sealed = self.session.simulation.cells[index].sealed; }
    }

    fn select_from_pointer(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        if !(82.0..=604.0).contains(&mouse_y) || mouse_x > 1_000.0 { return; }
        let ray = screen_ray(&self.camera.camera3d(), vec2(mouse_x, mouse_y), None);
        let mut closest: Option<(CellPos, f32)> = None;
        for y in 0..self.session.simulation.height { for x in 0..self.session.simulation.width {
            let pos = CellPos { x, y }; let cell = &self.session.simulation.cells[self.session.simulation.index(pos).unwrap()]; let height = (cell.height_hu as f32 * 0.0005).max(0.12);
            if let Some(distance) = Aabb3::from_center_size(vec3(x as f32 + 0.5, height * 0.5, y as f32 + 0.5), vec3(0.96, height, 0.96)).intersect(ray) { if closest.is_none_or(|(_, current)| distance < current) { closest = Some((pos, distance)); } }
        }}
        for device in &self.session.simulation.devices.devices { let (width, height) = device.device.footprint(); let cell = &self.session.simulation.cells[self.session.simulation.index(device.anchor).unwrap()]; let base = cell.height_hu as f32 * 0.0005; if let Some(distance) = Aabb3::from_center_size(vec3(device.anchor.x as f32 + width as f32 * 0.5, base + 0.35, device.anchor.y as f32 + height as f32 * 0.5), vec3(width as f32 * 0.72, 0.7, height as f32 * 0.72)).intersect(ray) { if closest.is_none_or(|(_, current)| distance < current) { closest = Some((device.anchor, distance)); } } }
        if let Some((selected, _)) = closest { self.session.selected = selected; let _ = self.session.mission.admit(CommandKind::Select); self.notice = format!("Survey target selected: {}, {}", selected.x, selected.y); }
    }

    fn queue_device(&mut self, device: DeviceId) {
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let result = devices.queue(&self.session.simulation, device, self.session.selected, 0, 100);
        self.session.simulation.devices = devices;
        self.notice = result.map(|id| format!("Queued {} plan #{id} — Enter commit, Backspace cancel", device.name())).unwrap_or_else(|error| format!("Queue rejected: {error:?}"));
    }

    fn commit_build_plan(&mut self) {
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let result = devices.commit_plan(&self.session.simulation, 100);
        self.session.simulation.devices = devices;
        self.notice = result.map(|entities| format!("Committed {} build plan(s)", entities.len())).unwrap_or_else(|error| format!("Plan commit rejected: {error:?}"));
    }

    fn cancel_build_plan(&mut self) {
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let cancelled = devices.cancel_last_plan();
        self.session.simulation.devices = devices;
        self.notice = if cancelled { "Last build plan cancelled" } else { "No queued build plan" }.into();
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
            let sim_cell = &self.session.simulation.cells[self.session.simulation.index(pos).unwrap()];
            let surface_depth = sim_cell.surface_volume() as f32 * 0.0005;
            if surface_depth > 0.0 {
                let surface_center = vec3(x as f32 + 0.5, h + surface_depth * 0.5 + 0.02, y as f32 + 0.5);
                let surface_color = sim_cell.surface.first().map(|material| fluid_color(material.fluid)).unwrap_or(WHITE);
                draw_cube(surface_center, vec3(0.88, surface_depth.max(0.04), 0.88), None, surface_color);
            }
            let steam_depth = sim_cell.airborne_volume() as f32 * 0.00035;
            if steam_depth > 0.0 { draw_cube(vec3(x as f32 + 0.5, h + 0.35 + steam_depth * 0.5, y as f32 + 0.5), vec3(0.7, steam_depth.max(0.08), 0.7), None, Color::new(0.76, 0.86, 0.92, 0.38)); }
        }}
        for device in &self.session.simulation.devices.devices {
            let (width, height) = device.device.footprint();
            let anchor = &self.session.simulation.cells[self.session.simulation.index(device.anchor).unwrap()];
            let center = vec3(device.anchor.x as f32 + width as f32 * 0.5, anchor.height_hu as f32 * 0.0005 + 0.35, device.anchor.y as f32 + height as f32 * 0.5);
            let color = if device.active { Color::new(0.35, 0.92, 0.72, 1.0) } else { Color::new(0.72, 0.52, 0.25, 1.0) };
            draw_cube(center, vec3(width as f32 * 0.72, 0.7, height as f32 * 0.72), None, color);
            draw_cube_wires(center, vec3(width as f32 * 0.78, 0.74, height as f32 * 0.78), if device.anchor == selected { WHITE } else { Color::new(0.08, 0.09, 0.12, 0.8) });
        }
        draw_grid_lines();
    }
}

fn fluid_color(fluid: FluidId) -> Color { match fluid { FluidId::Water => Color::new(0.12, 0.48, 0.9, 0.78), FluidId::Lava => Color::new(0.95, 0.22, 0.06, 0.9), FluidId::ToxicSlurry => Color::new(0.62, 0.72, 0.16, 0.86), FluidId::Steam => Color::new(0.76, 0.86, 0.92, 0.38) } }

fn draw_grid_lines() { /* Depth-tested cube silhouettes provide the stepped grid in R0. */ }
