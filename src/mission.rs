//! Mission lifecycle, campaign progression, and event-driven tutorial state.

use crate::{devices::DeviceId, simulation::SimulationWorld};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionId {
    L01FirstFlow,
    L02HoldingLine,
    L03Firebreak,
}

impl MissionId {
    pub const ALL: [Self; 3] = [Self::L01FirstFlow, Self::L02HoldingLine, Self::L03Firebreak];
    pub const fn sequence(self) -> usize {
        match self {
            Self::L01FirstFlow => 1,
            Self::L02HoldingLine => 2,
            Self::L03Firebreak => 3,
        }
    }
    pub const fn content_id(self) -> &'static str {
        match self {
            Self::L01FirstFlow => "campaign_l01_first_flow",
            Self::L02HoldingLine => "campaign_l02_holding_line",
            Self::L03Firebreak => "campaign_l03_firebreak",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionPhase {
    Briefing,
    Active,
    Success,
    Failure,
    Debrief,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Clear,
    Advisory,
    Warning,
    Critical,
}

impl AlertLevel {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Clear => "CLEAR",
            Self::Advisory => "ADVISORY",
            Self::Warning => "WARNING",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind {
    Camera,
    Select,
    Inspect,
    SetPaused,
    SelectTerrain,
    QueueExcavate,
    QueueDevice(DeviceId),
    CommitPlan,
    RecoverDeposit,
    ToggleDevice,
    SetTimeRunning,
    SetGate(u16),
    DismissPrompt,
    SkipTutorial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Admission {
    Accepted,
    TutorialLocked,
    MissionNotActive,
    AlreadyComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialState {
    pub tutorial_id: String,
    pub current_step_id: String,
    pub completed_step_ids: Vec<String>,
    pub hint_level: u8,
    pub ticks_in_step: u32,
    pub dismissed_prompt: bool,
    pub skip_confirmed: bool,
    #[serde(default)]
    pub actions_in_step: u8,
}

impl TutorialState {
    pub fn l01() -> Self {
        Self {
            tutorial_id: "tutorial_l01".into(),
            current_step_id: "tutorial_l01_welcome".into(),
            completed_step_ids: Vec::new(),
            hint_level: 0,
            ticks_in_step: 0,
            dismissed_prompt: false,
            skip_confirmed: false,
            actions_in_step: 0,
        }
    }
    pub fn skip(&mut self) {
        self.skip_confirmed = true;
        self.completed_step_ids = tutorial_steps();
        self.current_step_id = "tutorial_complete".into();
    }
    pub fn complete_current(&mut self) {
        if self.current_step_id != "tutorial_complete" {
            self.completed_step_ids.push(self.current_step_id.clone());
            let next = self.completed_step_ids.len();
            self.current_step_id = tutorial_steps()
                .get(next)
                .cloned()
                .unwrap_or_else(|| "tutorial_complete".into());
            self.ticks_in_step = 0;
            self.actions_in_step = 0;
            self.dismissed_prompt = false;
        }
    }
    pub fn is_complete(&self) -> bool {
        self.current_step_id == "tutorial_complete" || self.skip_confirmed
    }
}

fn tutorial_steps() -> Vec<String> {
    crate::content::ContentRegistry::load()
        .ok()
        .and_then(|content| {
            content
                .tutorials
                .into_iter()
                .find(|tutorial| tutorial.id == "tutorial_l01")
        })
        .map(|tutorial| tutorial.steps)
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionState {
    pub id: MissionId,
    pub phase: MissionPhase,
    pub tick: u64,
    pub stability_ticks: u32,
    pub objective_progress: u32,
    pub command_count: u32,
    pub failure_reason: Option<String>,
    pub tutorial: Option<TutorialState>,
    pub checkpoint_tick: u64,
    #[serde(default)]
    pub failure_ticks: u32,
    #[serde(default = "default_alert_level")]
    pub alert_level: AlertLevel,
    #[serde(default)]
    pub rules: MissionRules,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissionRules {
    pub objective_min_vu: u32,
    pub objective_max_vu: u32,
    pub stability_ticks: u32,
    pub failure_ticks: u32,
    pub advisory_tick: u64,
    pub warning_tick: u64,
    pub hazard_threshold_vu: u32,
    pub hazard_reason: String,
}

#[derive(Debug, Clone, Copy)]
struct MissionMetrics {
    basin_water: u32,
    trench_water: u32,
    shelf_rock: u32,
}

const fn default_alert_level() -> AlertLevel {
    AlertLevel::Clear
}

impl MissionState {
    pub fn new(id: MissionId) -> Self {
        let rules = rules_for(id);
        Self {
            id,
            phase: MissionPhase::Briefing,
            tick: 0,
            stability_ticks: 0,
            objective_progress: 0,
            command_count: 0,
            failure_reason: None,
            tutorial: (id == MissionId::L01FirstFlow).then(TutorialState::l01),
            checkpoint_tick: 0,
            failure_ticks: 0,
            alert_level: AlertLevel::Clear,
            rules,
        }
    }

    pub(crate) fn ensure_current_rules(&mut self) {
        if self.rules.stability_ticks == 0 {
            self.rules = rules_for(self.id);
        }
    }
    pub fn start(&mut self) {
        if self.phase == MissionPhase::Briefing {
            self.phase = MissionPhase::Active;
        }
    }
    pub fn admit(&mut self, command: CommandKind) -> Admission {
        if !matches!(self.phase, MissionPhase::Active) {
            return if matches!(
                self.phase,
                MissionPhase::Success | MissionPhase::Failure | MissionPhase::Debrief
            ) {
                Admission::AlreadyComplete
            } else {
                Admission::MissionNotActive
            };
        }
        if let Some(tutorial) = &mut self.tutorial {
            if !tutorial.skip_confirmed
                && !tutorial_allows(tutorial.current_step_id.as_str(), command)
            {
                return Admission::TutorialLocked;
            }
            advance_tutorial(tutorial, command);
        }
        self.command_count += 1;
        Admission::Accepted
    }
    pub fn skip_tutorial(&mut self) -> Admission {
        if self.phase != MissionPhase::Active {
            return Admission::MissionNotActive;
        }
        if let Some(tutorial) = &mut self.tutorial {
            tutorial.skip();
        }
        Admission::Accepted
    }
    pub fn checkpoint(&mut self) {
        self.checkpoint_tick = self.tick;
    }
    pub fn on_tick(&mut self, world: &SimulationWorld) {
        if self.phase != MissionPhase::Active {
            return;
        }
        self.tick = world.tick;
        self.advance_tutorial();
        let metrics = MissionMetrics {
            basin_water: zone_water(world, 26..=30, 7..=11),
            trench_water: zone_water(world, 31..=36, 7..=9),
            shelf_rock: zone_rock(world, 20..=25, 12..=16),
        };
        self.objective_progress = self.objective_progress(metrics);
        let hazard_active = self.hazard_active(world);
        self.update_alert(hazard_active);
        if self.failure_ticks >= self.rules.failure_ticks {
            self.fail(self.rules.hazard_reason.clone());
            return;
        }
        if self.objectives_met(world, metrics) {
            self.stability_ticks = self.stability_ticks.saturating_add(1);
        } else {
            self.stability_ticks = 0;
        }
        if self.stability_ticks >= self.rules.stability_ticks {
            self.phase = MissionPhase::Success;
        }
    }

    fn advance_tutorial(&mut self) {
        if let Some(tutorial) = &mut self.tutorial {
            tutorial.ticks_in_step = tutorial.ticks_in_step.saturating_add(1);
            if tutorial.current_step_id == "tutorial_l01_beacon_warning"
                && self.alert_level >= AlertLevel::Advisory
            {
                tutorial.complete_current();
            }
        }
    }

    fn objective_progress(&self, metrics: MissionMetrics) -> u32 {
        match self.id {
            MissionId::L01FirstFlow => metrics.basin_water,
            MissionId::L02HoldingLine => metrics.trench_water,
            MissionId::L03Firebreak => metrics.shelf_rock,
        }
    }

    fn hazard_active(&self, world: &SimulationWorld) -> bool {
        match self.id {
            MissionId::L01FirstFlow => {
                cell_water(world, crate::state::CellPos { x: 24, y: 8 })
                    >= self.rules.hazard_threshold_vu
            }
            MissionId::L02HoldingLine => {
                zone_water(world, 24..=28, 14..=18) >= self.rules.hazard_threshold_vu
            }
            MissionId::L03Firebreak => {
                zone_fluid(world, 35..=40, 12..=18, crate::simulation::FluidId::Lava) > 0
            }
        }
    }

    fn update_alert(&mut self, hazard_active: bool) {
        self.failure_ticks = if hazard_active {
            self.failure_ticks.saturating_add(1)
        } else {
            0
        };
        self.alert_level = match self.id {
            MissionId::L01FirstFlow if hazard_active => AlertLevel::Critical,
            MissionId::L01FirstFlow if self.objective_progress > 0 => AlertLevel::Advisory,
            MissionId::L02HoldingLine if hazard_active => AlertLevel::Critical,
            MissionId::L02HoldingLine
                if self.rules.warning_tick > 0 && self.tick >= self.rules.warning_tick =>
            {
                AlertLevel::Warning
            }
            MissionId::L02HoldingLine
                if self.rules.advisory_tick > 0 && self.tick >= self.rules.advisory_tick =>
            {
                AlertLevel::Advisory
            }
            MissionId::L03Firebreak if hazard_active => AlertLevel::Critical,
            MissionId::L03Firebreak if self.objective_progress > 0 => AlertLevel::Warning,
            MissionId::L03Firebreak
                if self.rules.advisory_tick > 0 && self.tick >= self.rules.advisory_tick =>
            {
                AlertLevel::Advisory
            }
            _ => AlertLevel::Clear,
        };
    }

    fn objectives_met(&self, world: &SimulationWorld, metrics: MissionMetrics) -> bool {
        match self.id {
            MissionId::L01FirstFlow => metrics.basin_water >= self.rules.objective_max_vu,
            MissionId::L02HoldingLine => {
                let reserve = world.devices.devices.iter().any(|device| {
                    device.device == DeviceId::Reservoir && device.stored_vu >= 2_000
                });
                (self.rules.objective_min_vu..=self.rules.objective_max_vu)
                    .contains(&metrics.trench_water)
                    && reserve
                    && self.tick >= self.rules.advisory_tick.saturating_add(650)
            }
            MissionId::L03Firebreak => {
                let turbine_power = world.devices.devices.iter().any(|device| {
                    device.device == DeviceId::FlowTurbine && device.cumulative_power >= 40
                });
                let relay_active = world
                    .devices
                    .devices
                    .iter()
                    .any(|device| device.device == DeviceId::RuneRelay && device.active);
                metrics.shelf_rock >= self.rules.objective_min_vu
                    && turbine_power
                    && relay_active
                    && self.tick >= self.rules.advisory_tick.saturating_add(550)
            }
        }
    }
    pub fn fail(&mut self, reason: impl Into<String>) {
        if self.phase == MissionPhase::Active {
            self.failure_reason = Some(reason.into());
            self.phase = MissionPhase::Failure;
        }
    }
}

fn rules_for(id: MissionId) -> MissionRules {
    crate::content::ContentRegistry::load()
        .ok()
        .and_then(|content| {
            content.mission(id.content_id()).map(|record| MissionRules {
                objective_min_vu: record.objective_min_vu,
                objective_max_vu: record.objective_max_vu,
                stability_ticks: record.stability_ticks,
                failure_ticks: record.failure_ticks,
                advisory_tick: record.advisory_tick,
                warning_tick: record.warning_tick,
                hazard_threshold_vu: record.hazard_threshold_vu,
                hazard_reason: record.hazard_reason.clone(),
            })
        })
        .unwrap_or_default()
}

fn cell_water(world: &SimulationWorld, pos: crate::state::CellPos) -> u32 {
    world
        .index(pos)
        .map(|index| {
            world.cells[index]
                .surface
                .iter()
                .filter(|entry| entry.fluid == crate::simulation::FluidId::Water)
                .map(|entry| entry.volume_vu)
                .sum()
        })
        .unwrap_or(0)
}
fn zone_water(
    world: &SimulationWorld,
    xs: std::ops::RangeInclusive<u16>,
    ys: std::ops::RangeInclusive<u16>,
) -> u32 {
    xs.flat_map(|x| ys.clone().map(move |y| crate::state::CellPos { x, y }))
        .map(|pos| cell_water(world, pos))
        .sum()
}
fn zone_fluid(
    world: &SimulationWorld,
    xs: std::ops::RangeInclusive<u16>,
    ys: std::ops::RangeInclusive<u16>,
    fluid: crate::simulation::FluidId,
) -> u32 {
    xs.flat_map(|x| ys.clone().map(move |y| crate::state::CellPos { x, y }))
        .map(|pos| {
            world
                .index(pos)
                .map(|index| {
                    world.cells[index]
                        .surface
                        .iter()
                        .filter(|entry| entry.fluid == fluid)
                        .map(|entry| entry.volume_vu)
                        .sum()
                })
                .unwrap_or(0)
        })
        .sum()
}

fn zone_rock(
    world: &SimulationWorld,
    xs: std::ops::RangeInclusive<u16>,
    ys: std::ops::RangeInclusive<u16>,
) -> u32 {
    xs.flat_map(|x| ys.clone().map(move |y| crate::state::CellPos { x, y }))
        .filter_map(|pos| world.index(pos).map(|index| &world.cells[index]))
        .map(|cell| {
            cell.pending_rock_vu
                + cell.pending_vitrified_vu
                + cell.formed_rock_vu
                + cell.formed_vitrified_vu
        })
        .sum()
}

fn tutorial_allows(step: &str, command: CommandKind) -> bool {
    match step {
        "tutorial_l01_welcome" => matches!(
            command,
            CommandKind::Camera
                | CommandKind::Select
                | CommandKind::Inspect
                | CommandKind::DismissPrompt
                | CommandKind::SkipTutorial
        ),
        "tutorial_l01_move_camera" => matches!(command, CommandKind::Camera),
        "tutorial_l01_move_cursor" => matches!(command, CommandKind::Select | CommandKind::Camera),
        "tutorial_l01_inspect_grade" => {
            matches!(command, CommandKind::Inspect | CommandKind::DismissPrompt)
        }
        "tutorial_l01_pause_plan" => matches!(command, CommandKind::SetPaused),
        "tutorial_l01_raise_first" | "tutorial_l01_raise_remaining" => {
            matches!(command, CommandKind::SelectTerrain)
        }
        "tutorial_l01_select_deposit" => matches!(command, CommandKind::Select),
        "tutorial_l01_recover_deposit" => matches!(command, CommandKind::RecoverDeposit),
        "tutorial_l01_place_channel" => {
            matches!(
                command,
                CommandKind::QueueDevice(DeviceId::Channel) | CommandKind::CommitPlan
            )
        }
        "tutorial_l01_run_and_observe" => {
            matches!(command, CommandKind::SetTimeRunning | CommandKind::Select)
        }
        "tutorial_l01_beacon_warning" => matches!(command, CommandKind::Inspect),
        "tutorial_l01_open_gate" => matches!(command, CommandKind::SetGate(10_000)),
        "tutorial_l01_stabilize" => {
            matches!(command, CommandKind::SetTimeRunning | CommandKind::Select)
        }
        _ => true,
    }
}
fn advance_tutorial(tutorial: &mut TutorialState, command: CommandKind) {
    if tutorial.current_step_id == "tutorial_l01_inspect_grade"
        && matches!(command, CommandKind::Inspect)
    {
        tutorial.dismissed_prompt = true;
        return;
    }
    let completes = matches!(
        (tutorial.current_step_id.as_str(), command),
        ("tutorial_l01_welcome", CommandKind::DismissPrompt)
            | ("tutorial_l01_move_camera", CommandKind::Camera)
            | ("tutorial_l01_move_cursor", CommandKind::Select)
            | ("tutorial_l01_select_deposit", CommandKind::Select)
            | ("tutorial_l01_recover_deposit", CommandKind::RecoverDeposit)
            | ("tutorial_l01_pause_plan", CommandKind::SetPaused)
            | ("tutorial_l01_raise_first", CommandKind::SelectTerrain)
            | ("tutorial_l01_raise_remaining", CommandKind::SelectTerrain)
            | ("tutorial_l01_place_channel", CommandKind::CommitPlan)
            | ("tutorial_l01_run_and_observe", CommandKind::SetTimeRunning)
            | ("tutorial_l01_beacon_warning", CommandKind::Inspect)
            | ("tutorial_l01_open_gate", CommandKind::SetGate(10_000))
            | ("tutorial_l01_stabilize", CommandKind::SetTimeRunning)
    ) || (tutorial.current_step_id == "tutorial_l01_inspect_grade"
        && tutorial.dismissed_prompt
        && matches!(command, CommandKind::DismissPrompt));
    if tutorial.current_step_id == "tutorial_l01_raise_remaining"
        && matches!(command, CommandKind::SelectTerrain)
    {
        tutorial.actions_in_step = tutorial.actions_in_step.saturating_add(1);
        if tutorial.actions_in_step < 2 {
            return;
        }
    }
    if completes {
        tutorial.complete_current();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub unlocked: [bool; 3],
    pub completed: [bool; 3],
    pub best_ticks: [Option<u64>; 3],
    pub content_version: String,
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self {
            unlocked: [true, false, false],
            completed: [false; 3],
            best_ticks: [None; 3],
            content_version: "1.2.0".into(),
        }
    }
}
impl CampaignProgress {
    pub fn record_success(&mut self, id: MissionId, ticks: u64) {
        let index = id.sequence() - 1;
        self.completed[index] = true;
        self.best_ticks[index] = Some(self.best_ticks[index].map_or(ticks, |best| best.min(ticks)));
        if index + 1 < 3 {
            self.unlocked[index + 1] = true;
        }
    }
}

pub fn campaign_summary() -> String {
    let content = crate::content::ContentRegistry::load().ok();
    MissionId::ALL
        .into_iter()
        .map(|id| {
            let title = content
                .as_ref()
                .and_then(|content| content.mission(id.content_id()))
                .map_or(id.content_id(), |mission| mission.title.as_str());
            format!("L0{} {title}", id.sequence())
        })
        .collect::<Vec<_>>()
        .join(" | ")
}
