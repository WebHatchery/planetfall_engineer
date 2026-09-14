//! Intent values emitted by controls before the game mutates authoritative state.

use crate::{
    devices::DeviceId,
    mission::MissionId,
    simulation::{FluidId, TerrainAction},
    state::{CellPos, TimeControl},
};

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
    if !(430.0..=810.0).contains(&x) {
        return None;
    }
    if (242.0..=274.0).contains(&y) {
        return if x < 540.0 {
            Some(UiAction::SetTime(TimeControl::OneX))
        } else if x < 660.0 {
            Some(UiAction::SetTime(TimeControl::TwoX))
        } else if x < 780.0 {
            Some(UiAction::SetTime(TimeControl::FourX))
        } else {
            None
        };
    }
    if (290.0..=322.0).contains(&y) {
        return if x < 555.0 {
            Some(UiAction::Save)
        } else if x < 680.0 {
            Some(UiAction::Load)
        } else if x < 810.0 {
            Some(UiAction::ResetMission)
        } else {
            None
        };
    }
    None
}
