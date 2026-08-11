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
    pub const fn name(self) -> &'static str {
        match self {
            Self::L01FirstFlow => "First Flow",
            Self::L02HoldingLine => "The Holding Line",
            Self::L03Firebreak => "Firebreak Protocol",
        }
    }
    pub const fn map_size(self) -> (u16, u16) {
        match self {
            Self::L01FirstFlow => (32, 20),
            Self::L02HoldingLine => (40, 24),
            Self::L03Firebreak => (48, 30),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        }
    }
    pub fn skip(&mut self) {
        self.skip_confirmed = true;
        self.completed_step_ids = tutorial_steps().iter().map(|step| (*step).into()).collect();
        self.current_step_id = "tutorial_complete".into();
    }
    pub fn complete_current(&mut self) {
        if self.current_step_id != "tutorial_complete" {
            self.completed_step_ids.push(self.current_step_id.clone());
            let next = self.completed_step_ids.len();
            self.current_step_id = tutorial_steps()
                .get(next)
                .copied()
                .unwrap_or("tutorial_complete")
                .into();
            self.ticks_in_step = 0;
            self.dismissed_prompt = false;
        }
    }
    pub fn is_complete(&self) -> bool {
        self.current_step_id == "tutorial_complete" || self.skip_confirmed
    }
}

fn tutorial_steps() -> [&'static str; 12] {
    [
        "tutorial_l01_welcome",
        "tutorial_l01_move_camera",
        "tutorial_l01_move_cursor",
        "tutorial_l01_inspect_grade",
        "tutorial_l01_pause_plan",
        "tutorial_l01_raise_first",
        "tutorial_l01_select_second",
        "tutorial_l01_raise_second",
        "tutorial_l01_select_third",
        "tutorial_l01_raise_third",
        "tutorial_l01_run_and_observe",
        "tutorial_l01_stabilize",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionState {
    pub id: MissionId,
    pub phase: MissionPhase,
    pub tick: u64,
    pub stability_ticks: u32,
    pub objective_progress: u32,
    pub budget: u32,
    pub command_count: u32,
    pub failure_reason: Option<String>,
    pub tutorial: Option<TutorialState>,
    pub checkpoint_tick: u64,
    #[serde(default)]
    pub failure_ticks: u32,
    #[serde(default = "default_alert_level")]
    pub alert_level: AlertLevel,
}

const fn default_alert_level() -> AlertLevel {
    AlertLevel::Clear
}

impl MissionState {
    pub fn new(id: MissionId) -> Self {
        Self {
            id,
            phase: MissionPhase::Briefing,
            tick: 0,
            stability_ticks: 0,
            objective_progress: 0,
            budget: match id {
                MissionId::L01FirstFlow => 40,
                MissionId::L02HoldingLine => 105,
                MissionId::L03Firebreak => 160,
            },
            command_count: 0,
            failure_reason: None,
            tutorial: (id == MissionId::L01FirstFlow).then(TutorialState::l01),
            checkpoint_tick: 0,
            failure_ticks: 0,
            alert_level: AlertLevel::Clear,
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
        if let Some(tutorial) = &mut self.tutorial {
            tutorial.ticks_in_step = tutorial.ticks_in_step.saturating_add(1);
        }
        let basin_water = zone_water(world, 26..=30, 7..=11);
        let trench_water = zone_water(world, 31..=36, 7..=9);
        let shelf_rock = zone_rock(world, 20..=25, 12..=16);
        self.objective_progress = match self.id {
            MissionId::L01FirstFlow => basin_water,
            MissionId::L02HoldingLine => trench_water,
            MissionId::L03Firebreak => shelf_rock,
        };
        let hazard_active = match self.id {
            MissionId::L01FirstFlow => {
                cell_water(world, crate::state::CellPos { x: 24, y: 8 }) >= 1_500
            }
            MissionId::L02HoldingLine => zone_water(world, 24..=28, 14..=18) >= 1_500,
            MissionId::L03Firebreak => {
                zone_fluid(world, 35..=40, 12..=18, crate::simulation::FluidId::Lava) > 0
            }
        };
        self.failure_ticks = if hazard_active {
            self.failure_ticks.saturating_add(1)
        } else {
            0
        };
        self.alert_level = match self.id {
            MissionId::L01FirstFlow if hazard_active => AlertLevel::Critical,
            MissionId::L01FirstFlow if self.objective_progress > 0 => AlertLevel::Advisory,
            MissionId::L02HoldingLine if hazard_active => AlertLevel::Critical,
            MissionId::L02HoldingLine if self.tick >= 800 => AlertLevel::Warning,
            MissionId::L02HoldingLine if self.tick >= 600 => AlertLevel::Advisory,
            MissionId::L03Firebreak if hazard_active => AlertLevel::Critical,
            MissionId::L03Firebreak if shelf_rock > 0 => AlertLevel::Warning,
            MissionId::L03Firebreak if self.tick >= 700 => AlertLevel::Advisory,
            _ => AlertLevel::Clear,
        };
        let failure_limit = if self.id == MissionId::L01FirstFlow {
            10
        } else {
            20
        };
        if self.failure_ticks >= failure_limit {
            self.fail(match self.id {
                MissionId::L01FirstFlow => "protected beacon flooded",
                MissionId::L02HoldingLine => "camp zone flooded",
                MissionId::L03Firebreak => "ancient foundation reached by lava",
            });
            return;
        }
        let objectives_met = match self.id {
            MissionId::L01FirstFlow => basin_water >= 6_000,
            MissionId::L02HoldingLine => {
                let reserve = world.devices.devices.iter().any(|device| {
                    device.device == DeviceId::Reservoir && device.stored_vu >= 2_000
                });
                (6_000..=9_000).contains(&trench_water) && reserve && self.tick >= 1_250
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
                shelf_rock >= 3_000 && turbine_power && relay_active && self.tick >= 1_250
            }
        };
        if objectives_met {
            self.stability_ticks = self.stability_ticks.saturating_add(1);
        } else {
            self.stability_ticks = 0;
        }
        let required = match self.id {
            MissionId::L01FirstFlow => 100,
            MissionId::L02HoldingLine => 150,
            MissionId::L03Firebreak => 150,
        };
        if self.stability_ticks >= required {
            self.phase = MissionPhase::Success;
        }
    }
    pub fn fail(&mut self, reason: impl Into<String>) {
        if self.phase == MissionPhase::Active {
            self.failure_reason = Some(reason.into());
            self.phase = MissionPhase::Failure;
        }
    }
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
        "tutorial_l01_raise_first" | "tutorial_l01_raise_second" | "tutorial_l01_raise_third" => {
            matches!(command, CommandKind::SelectTerrain)
        }
        "tutorial_l01_select_second" | "tutorial_l01_select_third" => {
            matches!(command, CommandKind::Select | CommandKind::Camera)
        }
        "tutorial_l01_run_and_observe" => {
            matches!(command, CommandKind::SetTimeRunning | CommandKind::Select)
        }
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
            | ("tutorial_l01_pause_plan", CommandKind::SetPaused)
            | ("tutorial_l01_raise_first", CommandKind::SelectTerrain)
            | ("tutorial_l01_select_second", CommandKind::Select)
            | ("tutorial_l01_raise_second", CommandKind::SelectTerrain)
            | ("tutorial_l01_select_third", CommandKind::Select)
            | ("tutorial_l01_raise_third", CommandKind::SelectTerrain)
            | ("tutorial_l01_run_and_observe", CommandKind::SetTimeRunning)
            | ("tutorial_l01_stabilize", CommandKind::SetTimeRunning)
    ) || (tutorial.current_step_id == "tutorial_l01_inspect_grade"
        && tutorial.dismissed_prompt
        && matches!(command, CommandKind::DismissPrompt));
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
            content_version: "phase-e-1".into(),
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
    MissionId::ALL
        .into_iter()
        .map(|id| format!("L0{} {}", id.sequence(), id.name()))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests;
