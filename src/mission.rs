//! Mission lifecycle, campaign progression, and event-driven tutorial state.

use crate::{devices::DeviceId, simulation::SimulationWorld};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionId { L01FirstFlow, L02HoldingLine, L03Firebreak }

impl MissionId {
    pub const ALL: [Self; 3] = [Self::L01FirstFlow, Self::L02HoldingLine, Self::L03Firebreak];
    pub const fn sequence(self) -> usize { match self { Self::L01FirstFlow => 1, Self::L02HoldingLine => 2, Self::L03Firebreak => 3 } }
    pub const fn name(self) -> &'static str { match self { Self::L01FirstFlow => "First Flow", Self::L02HoldingLine => "The Holding Line", Self::L03Firebreak => "Firebreak Protocol" } }
    pub const fn map_size(self) -> (u16, u16) { match self { Self::L01FirstFlow => (32, 20), Self::L02HoldingLine => (40, 24), Self::L03Firebreak => (48, 30) } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionPhase { Briefing, Active, Success, Failure, Debrief }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind { Camera, Select, Inspect, SetPaused, SelectTerrain, QueueExcavate, QueueDevice(DeviceId), CommitPlan, SetTimeRunning, SetGate(u16), DismissPrompt, SkipTutorial }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Admission { Accepted, TutorialLocked, MissionNotActive, AlreadyComplete }

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
    pub fn l01() -> Self { Self { tutorial_id: "tutorial_l01".into(), current_step_id: "tutorial_l01_welcome".into(), completed_step_ids: Vec::new(), hint_level: 0, ticks_in_step: 0, dismissed_prompt: false, skip_confirmed: false } }
    pub fn skip(&mut self) { self.skip_confirmed = true; self.completed_step_ids = tutorial_steps().iter().map(|step| (*step).into()).collect(); self.current_step_id = "tutorial_complete".into(); }
    pub fn complete_current(&mut self) { if self.current_step_id != "tutorial_complete" { self.completed_step_ids.push(self.current_step_id.clone()); let next = self.completed_step_ids.len(); self.current_step_id = tutorial_steps().get(next).copied().unwrap_or("tutorial_complete").into(); self.ticks_in_step = 0; self.dismissed_prompt = false; } }
    pub fn is_complete(&self) -> bool { self.current_step_id == "tutorial_complete" || self.skip_confirmed }
}

fn tutorial_steps() -> [&'static str; 12] { ["tutorial_l01_welcome", "tutorial_l01_move_camera", "tutorial_l01_move_cursor", "tutorial_l01_inspect_grade", "tutorial_l01_pause_plan", "tutorial_l01_excavate", "tutorial_l01_place_channel", "tutorial_l01_commit_plan", "tutorial_l01_run_and_observe", "tutorial_l01_control_gate", "tutorial_l01_see_impact", "tutorial_l01_stabilize"] }

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
}

impl MissionState {
    pub fn new(id: MissionId) -> Self { Self { id, phase: MissionPhase::Briefing, tick: 0, stability_ticks: 0, objective_progress: 0, budget: match id { MissionId::L01FirstFlow => 40, MissionId::L02HoldingLine => 105, MissionId::L03Firebreak => 160 }, command_count: 0, failure_reason: None, tutorial: (id == MissionId::L01FirstFlow).then(TutorialState::l01), checkpoint_tick: 0, failure_ticks: 0 } }
    pub fn start(&mut self) { if self.phase == MissionPhase::Briefing { self.phase = MissionPhase::Active; } }
    pub fn admit(&mut self, command: CommandKind) -> Admission {
        if !matches!(self.phase, MissionPhase::Active) { return if matches!(self.phase, MissionPhase::Success | MissionPhase::Failure | MissionPhase::Debrief) { Admission::AlreadyComplete } else { Admission::MissionNotActive }; }
        if let Some(tutorial) = &mut self.tutorial { if !tutorial.skip_confirmed && !tutorial_allows(tutorial.current_step_id.as_str(), command) { return Admission::TutorialLocked; } advance_tutorial(tutorial, command); }
        self.command_count += 1; Admission::Accepted
    }
    pub fn skip_tutorial(&mut self) -> Admission {
        if self.phase != MissionPhase::Active { return Admission::MissionNotActive; }
        if let Some(tutorial) = &mut self.tutorial { tutorial.skip(); }
        Admission::Accepted
    }
    pub fn checkpoint(&mut self) { self.checkpoint_tick = self.tick; }
    pub fn on_tick(&mut self, world: &SimulationWorld) {
        if self.phase != MissionPhase::Active { return; }
        self.tick = world.tick;
        if let Some(tutorial) = &mut self.tutorial { tutorial.ticks_in_step = tutorial.ticks_in_step.saturating_add(1); }
        let water = world.cells.iter().map(|cell| cell.surface.iter().filter(|m| m.fluid == crate::simulation::FluidId::Water).map(|m| m.volume_vu).sum::<u32>()).sum::<u32>();
        let formed_rock = world.cells.iter().map(|cell| cell.pending_rock_vu + cell.pending_vitrified_vu + cell.formed_rock_vu + cell.formed_vitrified_vu).sum::<u32>();
        self.objective_progress = if self.id == MissionId::L03Firebreak { formed_rock } else { water };
        let hazard_active = match self.id {
            MissionId::L01FirstFlow => cell_water(world, crate::state::CellPos { x: 10, y: 6 }) >= 1_500,
            MissionId::L02HoldingLine => zone_water(world, 24..=28, 14..=18) >= 1_500,
            MissionId::L03Firebreak => zone_fluid(world, 35..=40, 12..=18, crate::simulation::FluidId::Lava) > 0,
        };
        self.failure_ticks = if hazard_active { self.failure_ticks.saturating_add(1) } else { 0 };
        let failure_limit = if self.id == MissionId::L01FirstFlow { 10 } else { 20 };
        if self.failure_ticks >= failure_limit { self.fail(match self.id { MissionId::L01FirstFlow => "protected beacon flooded", MissionId::L02HoldingLine => "camp zone flooded", MissionId::L03Firebreak => "ancient foundation reached by lava" }); return; }
        let target = match self.id { MissionId::L01FirstFlow | MissionId::L02HoldingLine => 6_000, MissionId::L03Firebreak => 3_000 };
        if self.objective_progress >= target { self.stability_ticks = self.stability_ticks.saturating_add(1); } else { self.stability_ticks = 0; }
        let required = match self.id { MissionId::L01FirstFlow => 100, MissionId::L02HoldingLine => 150, MissionId::L03Firebreak => 150 };
        if self.stability_ticks >= required { self.phase = MissionPhase::Success; }
    }
    pub fn fail(&mut self, reason: impl Into<String>) { if self.phase == MissionPhase::Active { self.failure_reason = Some(reason.into()); self.phase = MissionPhase::Failure; } }
}

fn cell_water(world: &SimulationWorld, pos: crate::state::CellPos) -> u32 { world.index(pos).map(|index| world.cells[index].surface.iter().filter(|entry| entry.fluid == crate::simulation::FluidId::Water).map(|entry| entry.volume_vu).sum()).unwrap_or(0) }
fn zone_water(world: &SimulationWorld, xs: std::ops::RangeInclusive<u16>, ys: std::ops::RangeInclusive<u16>) -> u32 { xs.flat_map(|x| ys.clone().map(move |y| crate::state::CellPos { x, y })).map(|pos| cell_water(world, pos)).sum() }
fn zone_fluid(world: &SimulationWorld, xs: std::ops::RangeInclusive<u16>, ys: std::ops::RangeInclusive<u16>, fluid: crate::simulation::FluidId) -> u32 { xs.flat_map(|x| ys.clone().map(move |y| crate::state::CellPos { x, y })).map(|pos| world.index(pos).map(|index| world.cells[index].surface.iter().filter(|entry| entry.fluid == fluid).map(|entry| entry.volume_vu).sum()).unwrap_or(0)).sum() }

fn tutorial_allows(step: &str, command: CommandKind) -> bool { match step { "tutorial_l01_welcome" => matches!(command, CommandKind::Camera | CommandKind::Select | CommandKind::Inspect | CommandKind::DismissPrompt | CommandKind::SkipTutorial), "tutorial_l01_move_camera" => matches!(command, CommandKind::Camera | CommandKind::DismissPrompt), "tutorial_l01_move_cursor" => matches!(command, CommandKind::Select | CommandKind::Camera), "tutorial_l01_inspect_grade" => matches!(command, CommandKind::Inspect | CommandKind::DismissPrompt), "tutorial_l01_pause_plan" => matches!(command, CommandKind::SetPaused | CommandKind::SelectTerrain), "tutorial_l01_excavate" => matches!(command, CommandKind::QueueExcavate), "tutorial_l01_place_channel" => matches!(command, CommandKind::QueueDevice(DeviceId::Channel)), "tutorial_l01_commit_plan" => matches!(command, CommandKind::CommitPlan), "tutorial_l01_run_and_observe" => matches!(command, CommandKind::SetTimeRunning | CommandKind::Select), "tutorial_l01_control_gate" => matches!(command, CommandKind::SetPaused | CommandKind::SetGate(_)), "tutorial_l01_see_impact" => matches!(command, CommandKind::SetGate(_)), "tutorial_l01_stabilize" => matches!(command, CommandKind::SetTimeRunning), _ => true } }
fn advance_tutorial(tutorial: &mut TutorialState, command: CommandKind) { let completes = matches!((tutorial.current_step_id.as_str(), command), ("tutorial_l01_welcome", CommandKind::DismissPrompt) | ("tutorial_l01_move_camera", CommandKind::Camera) | ("tutorial_l01_move_cursor", CommandKind::Select) | ("tutorial_l01_inspect_grade", CommandKind::DismissPrompt) | ("tutorial_l01_pause_plan", CommandKind::SelectTerrain) | ("tutorial_l01_excavate", CommandKind::QueueExcavate) | ("tutorial_l01_place_channel", CommandKind::QueueDevice(DeviceId::Channel)) | ("tutorial_l01_commit_plan", CommandKind::CommitPlan) | ("tutorial_l01_run_and_observe", CommandKind::Select) | ("tutorial_l01_control_gate", CommandKind::SetGate(5_000)) | ("tutorial_l01_see_impact", CommandKind::SetGate(0)) | ("tutorial_l01_stabilize", CommandKind::SetTimeRunning)); if completes { tutorial.complete_current(); } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignProgress { pub unlocked: [bool; 3], pub completed: [bool; 3], pub best_ticks: [Option<u64>; 3], pub content_version: String }

impl Default for CampaignProgress { fn default() -> Self { Self { unlocked: [true, false, false], completed: [false; 3], best_ticks: [None; 3], content_version: "phase-e-1".into() } } }
impl CampaignProgress { pub fn record_success(&mut self, id: MissionId, ticks: u64) { let index = id.sequence() - 1; self.completed[index] = true; self.best_ticks[index] = Some(self.best_ticks[index].map_or(ticks, |best| best.min(ticks))); if index + 1 < 3 { self.unlocked[index + 1] = true; } } }

pub fn campaign_summary() -> String { MissionId::ALL.into_iter().map(|id| format!("L0{} {}", id.sequence(), id.name())).collect::<Vec<_>>().join(" | ") }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{FluidId, SimulationWorld};
    #[test] fn clean_campaign_unlocks_only_l01() { let progress = CampaignProgress::default(); assert_eq!(progress.unlocked, [true, false, false]); }
    #[test] fn tutorial_rejects_locked_state_changes_and_advances_on_events() { let mut mission = MissionState::new(MissionId::L01FirstFlow); mission.start(); assert_eq!(mission.admit(CommandKind::QueueDevice(DeviceId::Channel)), Admission::TutorialLocked); assert_eq!(mission.admit(CommandKind::DismissPrompt), Admission::Accepted); assert_eq!(mission.tutorial.as_ref().unwrap().current_step_id, "tutorial_l01_move_camera"); }
    #[test] fn skip_completes_tutorial_without_mutating_budget() { let mut mission = MissionState::new(MissionId::L01FirstFlow); mission.start(); let budget = mission.budget; assert_eq!(mission.skip_tutorial(), Admission::Accepted); assert!(mission.tutorial.as_ref().unwrap().is_complete()); assert_eq!(mission.budget, budget); }
    #[test] fn normal_tutorial_path_reaches_completion() { let mut mission = MissionState::new(MissionId::L01FirstFlow); mission.start(); for (index, command) in [CommandKind::DismissPrompt, CommandKind::Camera, CommandKind::Select, CommandKind::Inspect, CommandKind::DismissPrompt, CommandKind::SelectTerrain, CommandKind::QueueExcavate, CommandKind::QueueDevice(DeviceId::Channel), CommandKind::CommitPlan, CommandKind::Select, CommandKind::SetGate(5_000), CommandKind::SetGate(0), CommandKind::SetTimeRunning].into_iter().enumerate() { assert_eq!(mission.admit(command), Admission::Accepted, "step {index} current {:?}", mission.tutorial.as_ref().unwrap().current_step_id); } assert!(mission.tutorial.as_ref().unwrap().is_complete()); }
    #[test] fn success_requires_stability_and_terminal_state_stops_ticks() { let mut mission = MissionState::new(MissionId::L02HoldingLine); mission.start(); let mut world = SimulationWorld::new(2, 2); world.inject(crate::state::CellPos { x: 0, y: 0 }, FluidId::Water, 8_000); for _ in 0..150 { world.tick(); mission.on_tick(&world); } assert_eq!(mission.phase, MissionPhase::Success); let tick = mission.tick; mission.on_tick(&world); assert_eq!(mission.tick, tick); }
    #[test] fn firebreak_progress_uses_formed_rock() { let mut mission = MissionState::new(MissionId::L03Firebreak); mission.start(); let mut world = SimulationWorld::new(1, 1); world.inject(crate::state::CellPos { x: 0, y: 0 }, FluidId::Water, 4_000); world.inject(crate::state::CellPos { x: 0, y: 0 }, FluidId::Lava, 4_000); world.tick(); mission.on_tick(&world); assert_eq!(mission.objective_progress, 250); assert_eq!(mission.stability_ticks, 0); }
    #[test] fn authored_hazard_fails_before_success() { let mut mission = MissionState::new(MissionId::L01FirstFlow); mission.start(); let mut world = SimulationWorld::new(32, 20); let beacon = crate::state::CellPos { x: 10, y: 6 }; let index = world.index(beacon).unwrap(); world.cells[index].sealed = true; world.inject(beacon, FluidId::Water, 1_500); for _ in 0..10 { mission.on_tick(&world); } assert_eq!(mission.phase, MissionPhase::Failure); assert_eq!(mission.failure_reason.as_deref(), Some("protected beacon flooded")); }
    #[test] fn completion_unlocks_next_campaign_level_and_keeps_best_time() { let mut progress = CampaignProgress::default(); progress.record_success(MissionId::L01FirstFlow, 700); progress.record_success(MissionId::L01FirstFlow, 800); assert_eq!(progress.unlocked, [true, true, false]); assert_eq!(progress.best_ticks[0], Some(700)); }
}
