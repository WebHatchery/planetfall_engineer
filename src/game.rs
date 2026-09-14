//! Foundation orchestration: input, fixed ticks, orthographic world, HUD.

use crate::ui;
use crate::{
    campaign::load_campaign,
    data::GameData,
    devices::{DeviceId, SHOWCASE_MAPS},
    mission::{campaign_summary, CommandKind, MissionId},
    simulation::TerrainAction,
    state::{CellPos, GameSession, TimeControl, WorldState},
    verification::FluidsLab,
};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::ui::virtual_mouse_position;

pub struct Game {
    pub(crate) data: GameData,
    pub(crate) session: GameSession,
    pub(crate) checkpoint_session: Option<GameSession>,
    pub(crate) saved_campaign_session: Option<GameSession>,
    pub(crate) verification_mode: Option<VerificationMode>,
    pub(crate) verification_returns_to_menu: bool,
    pub(crate) frontend_mode: FrontendMode,
    pub(crate) _assets: AssetManager,
    pub(crate) terrain_texture: Texture2D,
    pub(crate) camera: FoundationCamera,
    pub(crate) lab: FluidsLab,
    pub(crate) notice: String,
    pub(crate) pause_menu: bool,
    pub(crate) placement_device: DeviceId,
    pub(crate) placement_rotation: u8,
    pub(crate) overlay_mode: u8,
}

mod actions;
mod camera;
mod picking;
mod update;
pub(crate) use camera::FoundationCamera;

#[derive(Debug, Clone, Copy)]
pub(crate) enum VerificationMode {
    Lab,
    Showcase(DeviceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendMode {
    Title,
    CampaignSelect,
    VerificationSelect,
    Playing,
}

fn make_terrain_texture() -> Texture2D {
    let mut image = Image::gen_image_color(32, 32, WHITE);
    for y in 0..32 {
        for x in 0..32 {
            let grain = ((x * 17 + y * 29 + (x ^ y) * 11) % 19) as f32 / 255.0;
            let seam = (x % 8 == 0 || y % 8 == 0) as u8 as f32 * 0.045;
            image.set_pixel(
                x,
                y,
                Color::new(
                    0.83 - grain - seam,
                    0.76 - grain - seam,
                    0.56 - grain * 0.7 - seam,
                    1.0,
                ),
            );
        }
    }
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
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
        // GameData is embedded and validated before the first frame; failure
        // here means the shipped binary cannot satisfy its startup contract.
        let data = GameData::load().expect("embedded foundation data must be valid");
        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(8, 8, Color::new(0.7, 0.25, 0.35, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let _ = assets.load_texture_configs(&data.texture_manifest).await;
        let terrain_texture = make_terrain_texture();
        let mut session = GameSession::new(&data.config);
        let campaign = load_campaign(MissionId::L01FirstFlow);
        let (width, height) = (campaign.world.width, campaign.world.height);
        session.simulation = campaign.world;
        session.world = WorldState::from_simulation(&session.simulation);
        let camera = FoundationCamera::new(data.config.world_width, data.config.world_height);
        let content_maps = data.content.maps.len();
        Self { data, session, checkpoint_session: None, saved_campaign_session: None, verification_mode: None, verification_returns_to_menu: false, frontend_mode: FrontendMode::Title, _assets: assets, terrain_texture, camera, lab: FluidsLab::new(), notice: format!("First Flow ready — {width}×{height} — FAB {} — reference {}–{} ticks — content {content_maps} maps validated", campaign.fabrication_start_fu, campaign.reference_tick_range.0, campaign.reference_tick_range.1), pause_menu: false, placement_device: DeviceId::Channel, placement_rotation: 0, overlay_mode: 0 }
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
            self.notice = "Failure fixture ready for recovery".into();
        }
        if scene.contains("pause") {
            self.pause_menu = true;
            self.session.time_control = TimeControl::Paused;
            self.notice = "Pause menu capture fixture".into();
        }
    }

    pub(crate) fn apply_terrain(&mut self, action: TerrainAction) {
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

    pub(crate) fn set_gate(&mut self, setting_bp: u16) {
        if !self.admit(CommandKind::SetGate(setting_bp)) {
            return;
        }
        let mut devices = std::mem::take(&mut self.session.simulation.devices);
        let changed = devices.set_selected_flow(self.session.selected, setting_bp);
        self.session.simulation.devices = devices;
        self.notice = if changed {
            format!("Selected flow control set to {}%", setting_bp / 100)
        } else {
            "Select a floodgate, pump, or reservoir".into()
        };
    }

    pub(crate) fn reset_mission(&mut self) {
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

    pub(crate) fn select_next_campaign(&mut self) {
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
        self.session.simulation = campaign.world;
        self.session.world = WorldState::from_simulation(&self.session.simulation);
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
        self.notice = format!("{} {reason}", self.mission_title(id));
    }

    pub(crate) fn mission_title(&self, id: MissionId) -> &str {
        self.data
            .content
            .mission(id.content_id())
            .map_or(id.content_id(), |mission| mission.title.as_str())
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
        let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        (490.0..=790.0).contains(&point.x) && (492.0..=542.0).contains(&point.y)
    }

    fn handle_tutorial_prompt_click(&mut self) -> bool {
        let Some(tutorial) = self.session.mission.tutorial.as_ref() else {
            return false;
        };
        let point = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        if (24.0..=188.0).contains(&point.x) && (202.0..=230.0).contains(&point.y) {
            self.skip_tutorial();
            return true;
        }
        if !matches!(
            tutorial.current_step_id.as_str(),
            "tutorial_l01_welcome" | "tutorial_l01_inspect_grade"
        ) {
            return false;
        }
        if (420.0..=524.0).contains(&point.x) && (114.0..=142.0).contains(&point.y) {
            let _ = self.admit(CommandKind::DismissPrompt);
            return true;
        }
        false
    }

    fn skip_tutorial(&mut self) {
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

    pub(crate) fn admit(&mut self, command: CommandKind) -> bool {
        match self.session.mission.admit(command) {
            crate::mission::Admission::Accepted => {
                let source_should_run =
                    self.session
                        .mission
                        .tutorial
                        .as_ref()
                        .is_some_and(|tutorial| {
                            tutorial.is_complete()
                                || tutorial.current_step_id == "tutorial_l01_stabilize"
                        });
                if source_should_run {
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
