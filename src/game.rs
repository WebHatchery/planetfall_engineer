//! Foundation orchestration: input, fixed ticks, orthographic world, HUD.

use crate::ui;
use crate::{
    campaign::{load_campaign, seed_reference_materials},
    data::GameData,
    devices::{run_all_showcases, DeviceId, SHOWCASE_MAPS},
    mission::{campaign_summary, CommandKind, MissionId, MissionPhase},
    replay::run_all_scenarios,
    simulation::{FluidId, TerrainAction},
    state::{load_session, save_session, CellPos, GameSession, TimeControl, WorldState},
    verification::FluidsLab,
};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::render3d::picking::{screen_ray, Aabb3};

pub struct Game {
    pub(crate) data: GameData,
    pub(crate) session: GameSession,
    pub(crate) checkpoint_session: Option<GameSession>,
    pub(crate) saved_campaign_session: Option<GameSession>,
    pub(crate) verification_mode: Option<VerificationMode>,
    pub(crate) verification_returns_to_menu: bool,
    pub(crate) frontend_mode: FrontendMode,
    pub(crate) assets: AssetManager,
    pub(crate) camera: FoundationCamera,
    pub(crate) lab: FluidsLab,
    pub(crate) notice: String,
    pub(crate) pause_menu: bool,
    pub(crate) placement_device: DeviceId,
    pub(crate) placement_rotation: u8,
    pub(crate) overlay_mode: u8,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum VerificationMode {
    Lab,
    Showcase(DeviceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrontendMode {
    Title,
    CampaignSelect,
    VerificationSelect,
    Playing,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FoundationCamera {
    target: Vec2,
    pub(crate) yaw: u8,
    pub(crate) zoom: f32,
}

impl FoundationCamera {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            target: vec2(width as f32 / 2.0, height as f32 / 2.0),
            yaw: 0,
            zoom: width.max(height) as f32,
        }
    }
    fn update(&mut self, dt: f32, width: usize, height: usize) -> bool {
        let before = (self.target, self.yaw, self.zoom);
        let speed = dt * self.zoom * 0.8;
        if is_key_down(KeyCode::A) {
            self.target.x -= speed;
        }
        if is_key_down(KeyCode::D) {
            self.target.x += speed;
        }
        if is_key_down(KeyCode::W) {
            self.target.y -= speed;
        }
        if is_key_down(KeyCode::S) {
            self.target.y += speed;
        }
        self.target.x = self.target.x.clamp(0.0, width as f32);
        self.target.y = self.target.y.clamp(0.0, height as f32);
        if is_key_pressed(KeyCode::Q) {
            self.yaw = (self.yaw + 3) % 4;
        }
        if is_key_pressed(KeyCode::E) {
            self.yaw = (self.yaw + 1) % 4;
        }
        if is_key_pressed(KeyCode::Minus) {
            self.zoom = (self.zoom + 4.0).min(54.0);
        }
        if is_key_pressed(KeyCode::Equal) {
            self.zoom = (self.zoom - 4.0).max(12.0);
        }
        before != (self.target, self.yaw, self.zoom)
    }
    pub(crate) fn camera3d(&self) -> Camera3D {
        let angle = self.yaw as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        let distance = self.zoom * 1.65;
        let target = vec3(self.target.x, 0.0, self.target.y);
        let viewport_scale = (screen_width() / ui::LOGICAL_WIDTH)
            .min(screen_height() / ui::LOGICAL_HEIGHT)
            .max(0.5);
        Camera3D {
            position: target
                + vec3(
                    angle.cos() * distance,
                    distance * 0.82,
                    angle.sin() * distance,
                ),
            target,
            up: vec3(0.0, 1.0, 0.0),
            projection: Projection::Orthographics,
            fovy: self.zoom / viewport_scale,
            aspect: Some(screen_width() / screen_height()),
            ..Default::default()
        }
    }
}

fn overlay_name(mode: u8) -> &'static str {
    match mode {
        1 => "GRADE",
        2 => "FLOW",
        3 => "HEAT",
        4 => "CONTAMINATION",
        _ => "MATERIAL",
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
        let content_maps = data.content.maps.len();
        Self { data, session, checkpoint_session: None, saved_campaign_session: None, verification_mode: None, verification_returns_to_menu: false, frontend_mode: FrontendMode::Title, assets, camera, lab: FluidsLab::new(), notice: format!("First Flow ready — {width}×{height} — budget {} — reference {}–{} ticks — content {content_maps} maps validated", campaign.budget, campaign.reference_tick_range.0, campaign.reference_tick_range.1), pause_menu: false, placement_device: DeviceId::Channel, placement_rotation: 0, overlay_mode: 0 }
    }

    pub fn begin_capture_scene(&mut self, scene: &str) {
        if scene.starts_with("title") {
            self.frontend_mode = FrontendMode::Title;
            return;
        }
        if scene.starts_with("campaign_board") {
            self.frontend_mode = FrontendMode::CampaignSelect;
            return;
        }
        if scene.starts_with("verification_grounds") {
            self.frontend_mode = FrontendMode::VerificationSelect;
            return;
        }
        self.frontend_mode = FrontendMode::Playing;
        if scene.starts_with("lab_fluids_all") {
            self.toggle_lab_mode();
        }
        if scene.contains("campaign_l01") {
            self.load_mission(MissionId::L01FirstFlow, "capture briefing");
        }
        if scene.contains("campaign_l02") {
            self.load_mission(MissionId::L02HoldingLine, "capture briefing");
        }
        if scene.contains("campaign_l03") {
            self.load_mission(MissionId::L03Firebreak, "capture briefing");
        }
        if scene.contains("active") {
            self.session.mission.start();
            self.session.time_control = TimeControl::OneX;
            self.notice = "Authored campaign source active — observe the live material loop".into();
        }
        if scene.contains("yaw2") {
            self.camera.yaw = 2;
            self.notice = "Opposing camera quarter capture".into();
        }
        if scene.contains("overlay_flow") {
            self.overlay_mode = 2;
            self.notice = "Flow overlay capture".into();
        }
        if let Some(showcase) = SHOWCASE_MAPS
            .iter()
            .find(|map| scene == map.map_id || scene.starts_with(&format!("{}_", map.map_id)))
        {
            self.enter_showcase(showcase.device);
        }
        if scene.contains("failure") {
            self.session
                .mission
                .fail("capture failure/recovery fixture");
            self.notice = "Failure fixture — F12 restores checkpoint or restarts".into();
        }
        if scene.contains("pause") {
            self.pause_menu = true;
            self.session.time_control = TimeControl::Paused;
            self.notice = "Pause menu capture fixture".into();
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.frontend_mode != FrontendMode::Playing {
            self.update_frontend();
            return;
        }
        if self.session.mission.phase == MissionPhase::Briefing {
            if is_key_pressed(KeyCode::Enter) || self.handle_briefing_click() {
                self.session.mission.start();
                self.notice = format!(
                    "{} operation active — follow the field guide and begin when ready",
                    self.session.mission.id.name()
                );
            }
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.pause_menu = !self.pause_menu;
            if self.pause_menu {
                self.session.time_control = TimeControl::Paused;
                self.notice = "Pause menu open — Escape resumes survey".into();
            }
        }
        if self.pause_menu {
            if is_key_pressed(KeyCode::F12) {
                self.reset_mission();
                self.pause_menu = false;
            } else if is_key_pressed(KeyCode::F5) {
                self.notice = save_session(&self.session, &self.data.config)
                    .map(|_| "Checkpoint saved from pause menu".into())
                    .unwrap_or_else(|e| e);
            } else if is_key_pressed(KeyCode::F9) {
                match load_session(&self.data.config) {
                    Ok(s) => {
                        self.session = s;
                        self.notice = "Checkpoint loaded from pause menu".into();
                    }
                    Err(e) => self.notice = e,
                }
            }
            return;
        }
        if self.camera.update(
            dt,
            self.session.simulation.width as usize,
            self.session.simulation.height as usize,
        ) {
            let _ = self.session.mission.admit(CommandKind::Camera);
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && !self.handle_palette_click()
            && !self.handle_build_action_click()
            && !self.handle_time_click()
            && !self.handle_verification_click()
        {
            self.select_from_pointer();
        }
        if is_key_pressed(KeyCode::Space) {
            self.set_time(match self.session.time_control {
                TimeControl::Paused => TimeControl::OneX,
                TimeControl::OneX => TimeControl::Paused,
                _ => TimeControl::Paused,
            });
        }
        if is_key_pressed(KeyCode::Key1) {
            self.set_time(TimeControl::OneX);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.set_time(TimeControl::TwoX);
        }
        if is_key_pressed(KeyCode::Key4) {
            self.set_time(TimeControl::FourX);
        }
        let mut cursor_moved = false;
        if is_key_pressed(KeyCode::Up) {
            self.session.move_selected(0, -1);
            cursor_moved = true;
        }
        if is_key_pressed(KeyCode::Down) {
            self.session.move_selected(0, 1);
            cursor_moved = true;
        }
        if is_key_pressed(KeyCode::Left) {
            self.session.move_selected(-1, 0);
            cursor_moved = true;
        }
        if is_key_pressed(KeyCode::Right) {
            self.session.move_selected(1, 0);
            cursor_moved = true;
        }
        if cursor_moved {
            let _ = self.session.mission.admit(CommandKind::Select);
        }
        if is_key_pressed(KeyCode::X) {
            self.apply_terrain(TerrainAction::Excavate);
        }
        if is_key_pressed(KeyCode::R) {
            self.apply_terrain(TerrainAction::Raise);
        }
        if is_key_pressed(KeyCode::T) {
            self.apply_terrain(TerrainAction::Seal);
        }
        if is_key_pressed(KeyCode::I) && self.admit(CommandKind::Inspect) {
            self.notice = format!(
                "Inspecting cell {}, {}",
                self.session.selected.x, self.session.selected.y
            );
        }
        if is_key_pressed(KeyCode::L) {
            self.session
                .simulation
                .inject(self.session.selected, FluidId::Lava, 500);
        }
        if is_key_pressed(KeyCode::G) {
            self.session
                .simulation
                .inject(self.session.selected, FluidId::ToxicSlurry, 500);
        }
        if is_key_pressed(KeyCode::Y) {
            self.overlay_mode = (self.overlay_mode + 1) % 5;
            self.notice = format!("{} overlay", overlay_name(self.overlay_mode));
        }
        if is_key_pressed(KeyCode::F1) {
            self.toggle_lab_mode();
        }
        if is_key_pressed(KeyCode::F2) {
            if self.verification_mode.is_some() {
                self.restore_campaign_session();
            } else {
                self.enter_showcase(DeviceId::Channel);
            }
        }
        if is_key_pressed(KeyCode::V) {
            if let Some(VerificationMode::Showcase(current)) = self.verification_mode {
                let next = DeviceId::ALL[(DeviceId::ALL
                    .iter()
                    .position(|device| *device == current)
                    .unwrap_or(0)
                    + 1)
                    % DeviceId::ALL.len()];
                self.enter_showcase(next);
            }
        }
        if is_key_pressed(KeyCode::F3) {
            self.notice = campaign_summary();
        }
        if is_key_pressed(KeyCode::N) {
            self.select_next_campaign();
        }
        if is_key_pressed(KeyCode::F4) {
            self.session.mission.skip_tutorial();
            let complete = self
                .session
                .mission
                .tutorial
                .as_ref()
                .is_some_and(|tutorial| tutorial.is_complete());
            if complete {
                self.session.simulation.set_sources_enabled(true);
            }
            self.notice = if complete {
                "Tutorial skipped — L01 build kit unlocked; source enabled"
            } else {
                "Tutorial skip unavailable"
            }
            .into();
        }
        if is_key_pressed(KeyCode::F6) && self.verification_mode.is_some() {
            self.reset_verification();
        } else if is_key_pressed(KeyCode::F6) {
            self.session.mission.checkpoint();
            self.checkpoint_session = Some(self.session.clone());
            self.notice = format!(
                "Mission checkpoint recorded at tick {}",
                self.session.mission.checkpoint_tick
            );
        }
        if is_key_pressed(KeyCode::F7) {
            self.session.mission.fail("manual failure-path check");
            self.notice = "Mission failed — reset to checkpoint".into();
        }
        if is_key_pressed(KeyCode::F12) {
            self.reset_mission();
        }
        if is_key_pressed(KeyCode::F8) && self.verification_mode.is_some() {
            self.step_verification();
        } else if is_key_pressed(KeyCode::F8) {
            let admission = self.session.mission.admit(CommandKind::DismissPrompt);
            self.notice = format!("{} — tutorial command: {admission:?}", run_all_showcases());
        }
        if is_key_pressed(KeyCode::F10) {
            let mut campaign = load_campaign(self.session.mission.id);
            seed_reference_materials(&mut campaign);
            self.session.simulation = campaign.world;
            self.notice = "Reference material fixture loaded".into();
        }
        if is_key_pressed(KeyCode::F11) {
            self.notice = run_all_scenarios();
        }
        if is_key_pressed(KeyCode::C) {
            if let Some(entity_id) = self
                .session
                .simulation
                .devices
                .devices
                .iter()
                .find(|device| device.anchor == self.session.selected)
                .map(|device| device.entity_id)
            {
                self.session.simulation.devices.remove(entity_id);
                self.notice = "Device removed and budget released".into();
            }
        }
        if is_key_pressed(KeyCode::B) {
            self.queue_device(DeviceId::Channel);
        }
        if is_key_pressed(KeyCode::P) {
            self.queue_device(DeviceId::Pipe);
        }
        if is_key_pressed(KeyCode::O) {
            self.queue_device(DeviceId::Pump);
        }
        if is_key_pressed(KeyCode::F) {
            self.queue_device(DeviceId::Floodgate);
        }
        if is_key_pressed(KeyCode::Z) {
            self.placement_rotation = (self.placement_rotation + 1) % 4;
            self.notice = format!(
                "Placement rotation {}° — {} at {}, {}",
                self.placement_rotation * 90,
                self.placement_device.name(),
                self.session.selected.x,
                self.session.selected.y
            );
        }
        if is_key_pressed(KeyCode::J) {
            self.set_gate(0);
        }
        if is_key_pressed(KeyCode::K) {
            self.set_gate(5_000);
        }
        if is_key_pressed(KeyCode::H) {
            self.set_gate(10_000);
        }
        if is_key_pressed(KeyCode::Enter) {
            if self.session.simulation.devices.queued.is_empty() {
                let _ = self.admit(CommandKind::DismissPrompt);
            } else {
                self.commit_build_plan();
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.cancel_build_plan();
        }
        if is_key_pressed(KeyCode::F5) {
            self.notice = save_session(&self.session, &self.data.config)
                .map(|_| "Checkpoint saved".into())
                .unwrap_or_else(|e| e);
        }
        if is_key_pressed(KeyCode::F9) {
            match load_session(&self.data.config) {
                Ok(s) => {
                    self.session = s;
                    self.notice = "Checkpoint loaded".into();
                }
                Err(e) => self.notice = e,
            }
        }
        let ticks = self.session.update(dt);
        if ticks > 0 {
            self.session.mission.on_tick(&self.session.simulation);
            if self.session.mission.phase == MissionPhase::Success {
                self.session
                    .campaign
                    .record_success(self.session.mission.id, self.session.mission.tick);
                self.notice = format!(
                    "Mission success — {} complete; press N for the next unlocked level",
                    self.session.mission.id.name()
                );
            } else {
                self.notice = format!("Simulation advanced {ticks} tick(s)");
            }
        }
    }

    fn apply_terrain(&mut self, action: TerrainAction) {
        let command = if action == TerrainAction::Excavate {
            CommandKind::QueueExcavate
        } else {
            CommandKind::SelectTerrain
        };
        if !self.admit(command) {
            return;
        }
        self.notice = self
            .session
            .simulation
            .terrain_edit(self.session.selected, action)
            .map(|_| "Terrain edit committed".into())
            .unwrap_or_else(|error| format!("Terrain edit rejected: {error:?}"));
        if let Some(index) = self.session.simulation.index(self.session.selected) {
            self.session.world.cells[index].height_hu =
                self.session.simulation.cells[index].height_hu;
            self.session.world.cells[index].sealed = self.session.simulation.cells[index].sealed;
        }
    }

    fn select_from_pointer(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        if !(82.0..=604.0).contains(&mouse_y) || mouse_x > 1_000.0 {
            return;
        }
        let ray = screen_ray(&self.camera.camera3d(), vec2(mouse_x, mouse_y), None);
        let mut closest: Option<(CellPos, f32)> = None;
        for y in 0..self.session.simulation.height {
            for x in 0..self.session.simulation.width {
                let pos = CellPos { x, y };
                let cell =
                    &self.session.simulation.cells[self.session.simulation.index(pos).unwrap()];
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
            let cell = &self.session.simulation.cells
                [self.session.simulation.index(device.anchor).unwrap()];
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
            self.session.selected = selected;
            let _ = self.session.mission.admit(CommandKind::Select);
            self.notice = format!("Survey target selected: {}, {}", selected.x, selected.y);
        }
    }

    fn set_gate(&mut self, setting_bp: u16) {
        if !self.admit(CommandKind::SetGate(setting_bp)) {
            return;
        }
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let changed = devices.set_selected_gate(self.session.selected, setting_bp);
        self.session.simulation.devices = devices;
        self.notice = if changed {
            format!("Floodgate set to {}%", setting_bp / 100)
        } else {
            "No floodgate selected".into()
        };
    }

    fn reset_mission(&mut self) {
        if let Some(checkpoint) = self.checkpoint_session.clone() {
            let campaign = self.session.campaign.clone();
            self.session = checkpoint;
            self.session.campaign = campaign;
            self.session.time_control = TimeControl::Paused;
            self.notice = format!(
                "Checkpoint restored at tick {}",
                self.session.mission.checkpoint_tick
            );
            return;
        }
        let id = self.session.mission.id;
        self.load_mission(id, "restarted from briefing");
    }

    fn select_next_campaign(&mut self) {
        let current = self.session.mission.id.sequence();
        let next = MissionId::ALL.into_iter().find(|id| {
            id.sequence() > current && self.session.campaign.unlocked[id.sequence() - 1]
        });
        match next {
            Some(id) => self.load_mission(id, "selected from campaign progression"),
            None => {
                self.notice = format!(
                    "No later campaign level is unlocked — {}",
                    campaign_summary()
                )
            }
        }
    }

    pub(crate) fn load_mission(&mut self, id: MissionId, reason: &str) {
        let campaign_progress = self.session.campaign.clone();
        let campaign = load_campaign(id);
        let (width, height) = (
            campaign.world.width as usize,
            campaign.world.height as usize,
        );
        self.session = GameSession::new(&self.data.config);
        self.session.world = WorldState::new(width, height);
        self.session.simulation = campaign.world;
        self.session
            .simulation
            .set_sources_enabled(id != MissionId::L01FirstFlow);
        self.session.mission = crate::mission::MissionState::new(id);
        self.session.campaign = campaign_progress;
        self.session.selected = CellPos {
            x: (width / 2) as u16,
            y: (height / 2) as u16,
        };
        self.session.time_control = TimeControl::Paused;
        self.camera = FoundationCamera::new(width, height);
        self.checkpoint_session = None;
        self.frontend_mode = FrontendMode::Playing;
        self.notice = format!("{} {reason}", id.name());
    }

    pub(crate) fn set_time(&mut self, time: TimeControl) {
        let command = if time == TimeControl::Paused {
            CommandKind::SetPaused
        } else {
            CommandKind::SetTimeRunning
        };
        if self.admit(command) {
            self.session.time_control = time;
        }
    }

    fn handle_briefing_click(&self) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        let (mouse_x, mouse_y) = mouse_position();
        let x = mouse_x / (screen_width() / ui::LOGICAL_WIDTH);
        let y = mouse_y / (screen_height() / ui::LOGICAL_HEIGHT);
        (490.0..=790.0).contains(&x) && (492.0..=542.0).contains(&y)
    }

    pub(crate) fn admit(&mut self, command: CommandKind) -> bool {
        match self.session.mission.admit(command) {
            crate::mission::Admission::Accepted => {
                if self
                    .session
                    .mission
                    .tutorial
                    .as_ref()
                    .is_some_and(|tutorial| tutorial.is_complete())
                {
                    self.session.simulation.set_sources_enabled(true);
                }
                true
            }
            result => {
                self.notice = format!("Command unavailable: {result:?}");
                false
            }
        }
    }
}
