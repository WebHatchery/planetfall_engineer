//! Intent values emitted by controls before the game mutates authoritative state.

use crate::{
    devices::DeviceId,
    mission::MissionId,
    simulation::{FluidId, TerrainAction},
    state::{CellPos, TimeControl},
};
use macroquad::prelude::*;
use macroquad_toolkit::input::{hit_test, HitTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    TogglePause,
    SetTime(TimeControl),
    Select(CellPos),
    MoveSelection(i16, i16),
    Inspect,
    Terrain(TerrainAction),
    QueueDevice(DeviceId),
    RotatePlacement,
    CommitPlan,
    CancelPlan,
    SetGate(u16),
    RecoverDeposit,
    ToggleSelectedDevice,
    Save,
    Load,
    ResetMission,
    BeginMission,
    SkipTutorial,
    ToggleLab,
    EnterShowcase(DeviceId),
    RestoreCampaign,
    StepVerification,
    NextCampaign,
    StartNewCampaign,
    OpenCampaignSelect,
    OpenVerificationSelect,
    SelectMission(MissionId),
    TerminalPrimary,
    TerminalSecondary,
    ResetVerification,
    RemoveSelectedDevice,
    Checkpoint,
    FailMission,
    ReloadMission,
    Inject(FluidId),
    CycleOverlay,
    RunShowcaseReport,
    RunScenarioReport,
    DismissPrompt,
}

pub fn pause_action_at(x: f32, y: f32) -> Option<UiAction> {
    hit_test(
        [
            HitTarget::new(
                Rect::new(430.0, 242.0, 110.0, 32.0),
                UiAction::SetTime(TimeControl::OneX),
            ),
            HitTarget::new(
                Rect::new(550.0, 242.0, 110.0, 32.0),
                UiAction::SetTime(TimeControl::TwoX),
            ),
            HitTarget::new(
                Rect::new(670.0, 242.0, 110.0, 32.0),
                UiAction::SetTime(TimeControl::FourX),
            ),
            HitTarget::new(Rect::new(430.0, 290.0, 125.0, 32.0), UiAction::Save),
            HitTarget::new(Rect::new(555.0, 290.0, 125.0, 32.0), UiAction::Load),
            HitTarget::new(Rect::new(680.0, 290.0, 125.0, 32.0), UiAction::ResetMission),
        ],
        vec2(x, y),
    )
}
