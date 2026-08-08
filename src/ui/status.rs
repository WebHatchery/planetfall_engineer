//! Compact player-facing mission and selected-device status strings.

use crate::{devices::DeviceId, state::GameSession};

pub(super) fn mission_primary_status(session: &GameSession) -> String {
    match session.mission.id {
        crate::mission::MissionId::L01FirstFlow => {
            format!("Basin {} / 6000 vU", session.mission.objective_progress)
        }
        crate::mission::MissionId::L02HoldingLine => {
            format!("Trench {} / 6000-9000", session.mission.objective_progress)
        }
        crate::mission::MissionId::L03Firebreak => {
            format!(
                "Firebreak {} / 3000 rock",
                session.mission.objective_progress
            )
        }
    }
}

pub(super) fn mission_secondary_status(session: &GameSession) -> String {
    match session.mission.id {
        crate::mission::MissionId::L01FirstFlow => {
            format!("Stable {} / 100 ticks", session.mission.stability_ticks)
        }
        crate::mission::MissionId::L02HoldingLine => {
            let reserve: u32 = session
                .simulation
                .devices
                .devices
                .iter()
                .filter(|device| device.device == DeviceId::Reservoir)
                .map(|device| device.stored_vu)
                .sum();
            format!(
                "Reserve {reserve}/2000 | Surge {}/1250",
                session.mission.tick.min(1_250)
            )
        }
        crate::mission::MissionId::L03Firebreak => {
            let power: u64 = session
                .simulation
                .devices
                .devices
                .iter()
                .filter(|device| device.device == DeviceId::FlowTurbine)
                .map(|device| device.cumulative_power)
                .sum();
            let relay = session
                .simulation
                .devices
                .devices
                .iter()
                .any(|device| device.device == DeviceId::RuneRelay && device.active);
            format!(
                "Power {power}/40 | Relay {}",
                if relay { "ACTIVE" } else { "OFF" }
            )
        }
    }
}

pub(super) fn device_status(device: &crate::devices::DeviceState) -> String {
    let state = if device.active { "ACTIVE" } else { "IDLE" };
    match device.device {
        DeviceId::Reservoir => format!(
            "RES {}vU HOLD {}% {state}",
            device.stored_vu,
            device.setting_bp / 100
        ),
        DeviceId::FlowTurbine => {
            format!("TURBINE {} power {state}", device.cumulative_power)
        }
        DeviceId::RuneRelay => format!(
            "RELAY PWR {} FLOW {}vU {state}",
            if device.powered { "YES" } else { "NO" },
            device.stored_vu
        ),
        _ => format!(
            "{} {}% {state}",
            device.device.name(),
            device.setting_bp / 100
        ),
    }
}
