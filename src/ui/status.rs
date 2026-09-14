//! Compact player-facing mission and selected-device status strings.

use crate::{content::ContentRegistry, devices::DeviceId, state::GameSession};

pub(super) fn mission_primary_status(session: &GameSession, content: &ContentRegistry) -> String {
    let Some(mission) = content.mission(session.mission.id.content_id()) else {
        return format!("Progress {}", session.mission.objective_progress);
    };
    let label = match session.mission.id {
        crate::mission::MissionId::L01FirstFlow => "Dam",
        crate::mission::MissionId::L02HoldingLine => "Trench",
        crate::mission::MissionId::L03Firebreak => "Firebreak",
    };
    if mission.objective_min_vu == mission.objective_max_vu {
        format!(
            "{label} {} / {} vU",
            session.mission.objective_progress, mission.objective_min_vu
        )
    } else {
        format!(
            "{label} {} / {}-{}",
            session.mission.objective_progress, mission.objective_min_vu, mission.objective_max_vu
        )
    }
}

pub(super) fn mission_secondary_status(session: &GameSession, content: &ContentRegistry) -> String {
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
            let surge_end = content
                .mission(session.mission.id.content_id())
                .and_then(|mission| mission.surge_end_tick)
                .unwrap_or(session.mission.tick);
            format!(
                "Reserve {reserve}/2000 | Surge {}/{}",
                session.mission.tick.min(surge_end),
                surge_end
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
