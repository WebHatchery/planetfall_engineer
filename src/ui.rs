//! Screen-space engineering HUD restored after the 3D world pass.

use crate::{
    mission::MissionPhase,
    simulation::FluidId,
    state::{GameSession, TimeControl},
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

pub struct UiContext<'a> {
    pub session: &'a GameSession,
    pub camera_yaw: u8,
    pub camera_zoom: f32,
    pub notice: &'a str,
    pub loaded_assets: usize,
    pub verification_label: Option<&'a str>,
    pub pause_menu: bool,
    pub placement_device: crate::devices::DeviceId,
    pub placement_rotation: u8,
    pub placement_valid: bool,
    pub placement_reason: &'a str,
    pub overlay_mode: u8,
}

pub fn draw_hud(ctx: UiContext<'_>) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        82.0,
        Color::new(0.035, 0.05, 0.08, 0.96),
    );
    draw_ui_text_ex(
        "PLANETFALL ENGINEER",
        28.0,
        39.0,
        TextStyle::new(28.0, Color::new(0.92, 0.86, 0.68, 1.0)).params(),
    );
    let heading = ctx
        .verification_label
        .map(str::to_owned)
        .unwrap_or_else(|| {
            format!(
                "L0{} // {} // {}",
                ctx.session.mission.id.sequence(),
                ctx.session.mission.id.name().to_uppercase(),
                phase_name(ctx.session.mission.phase)
            )
        });
    draw_ui_text_ex(
        &heading,
        30.0,
        64.0,
        TextStyle::new(13.0, Color::new(0.48, 0.62, 0.72, 1.0)).params(),
    );
    draw_ui_text_ex(
        &format!("OVERLAY {}", overlay_name(ctx.overlay_mode)),
        30.0,
        78.0,
        TextStyle::new(12.0, Color::new(0.5, 0.7, 0.72, 1.0)).params(),
    );
    draw_text_right(
        &format!(
            "TICK {:04}   HASH {:016X}",
            ctx.session.tick,
            ctx.session.state_hash()
        ),
        1248.0,
        38.0,
        TextStyle::new(16.0, WHITE),
    );
    draw_text_right(
        &format!(
            "TIME {}   YAW {}   ZOOM {}",
            time_name(ctx.session.time_control),
            u32::from(ctx.camera_yaw) * 90,
            ctx.camera_zoom as u32
        ),
        1248.0,
        62.0,
        TextStyle::new(14.0, Color::new(0.65, 0.75, 0.82, 1.0)),
    );
    draw_rectangle(
        24.0,
        604.0,
        1232.0,
        88.0,
        Color::new(0.04, 0.06, 0.09, 0.96),
    );
    draw_ui_text_ex(
        &format!(
            "SURVEY CURSOR  {}, {}",
            ctx.session.selected.x, ctx.session.selected.y
        ),
        44.0,
        634.0,
        TextStyle::new(18.0, Color::new(0.95, 0.8, 0.35, 1.0)).params(),
    );
    draw_ui_text_ex(
        ctx.notice,
        44.0,
        660.0,
        TextStyle::new(15.0, Color::new(0.7, 0.76, 0.8, 1.0)).params(),
    );
    draw_ui_text_ex(
        "Tap the map to survey. Use the engineering controls at right to build and manage time.",
        44.0,
        682.0,
        TextStyle::new(13.0, Color::new(0.5, 0.58, 0.64, 1.0)).params(),
    );
    draw_rectangle(
        1010.0,
        104.0,
        238.0,
        420.0,
        Color::new(0.04, 0.06, 0.09, 0.9),
    );
    draw_ui_text_ex(
        "ENGINEERING READOUT",
        1024.0,
        130.0,
        TextStyle::new(14.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
    );
    let selected =
        &ctx.session.simulation.cells[ctx.session.simulation.index(ctx.session.selected).unwrap()];
    let selected_definition = &ctx.session.simulation.definitions
        [ctx.session.simulation.index(ctx.session.selected).unwrap()];
    let heat_dk = selected
        .surface
        .first()
        .map(|material| material.temperature_dk)
        .unwrap_or(selected_definition.ambient_temperature_dk);
    let device_readout = ctx
        .session
        .simulation
        .devices
        .devices
        .iter()
        .find(|device| device.anchor == ctx.session.selected)
        .map(|device| {
            format!(
                "Device {} {}% {}",
                device.device.name(),
                device.setting_bp / 100,
                if device.active { "ACTIVE" } else { "IDLE" }
            )
        })
        .unwrap_or_else(|| "Device none".into());
    let tutorial = ctx
        .session
        .mission
        .tutorial
        .as_ref()
        .map(|tutorial| {
            tutorial
                .current_step_id
                .strip_prefix("tutorial_l01_")
                .unwrap_or(&tutorial.current_step_id)
        })
        .unwrap_or("none");
    let style = TextStyle::new(13.0, Color::new(0.7, 0.76, 0.8, 1.0));
    draw_ui_text_ex(
        &format!(
            "Map {}×{}",
            ctx.session.simulation.width, ctx.session.simulation.height
        ),
        1024.0,
        154.0,
        style.params(),
    );
    let objective = ctx
        .verification_label
        .map(|label| format!("{} tick {}", label, ctx.session.simulation.tick))
        .unwrap_or_else(|| {
            format!(
                "Objective {} / {} vU",
                ctx.session.mission.objective_progress,
                objective_target(ctx.session.mission.id)
            )
        });
    let objective = if ctx.verification_label.is_some() {
        objective
    } else {
        format!(
            "{}  STAB {}/{}",
            objective,
            ctx.session.mission.stability_ticks,
            stability_target(ctx.session.mission.id)
        )
    };
    draw_ui_text_ex(&objective, 1024.0, 172.0, style.params());
    draw_ui_text_ex(
        &format!("ALERT {}", ctx.session.mission.alert_level.name()),
        1024.0,
        190.0,
        TextStyle::new(
            13.0,
            if ctx.session.mission.alert_level == crate::mission::AlertLevel::Critical {
                Color::new(0.95, 0.45, 0.38, 1.0)
            } else {
                Color::new(0.95, 0.8, 0.35, 1.0)
            },
        )
        .params(),
    );
    draw_ui_text_ex(
        &format!("Tutorial {}", tutorial),
        1024.0,
        208.0,
        style.params(),
    );
    draw_ui_text_ex(
        &format!(
            "Assets {}   Fluids {}",
            ctx.loaded_assets,
            FluidId::ALL.len()
        ),
        1024.0,
        226.0,
        style.params(),
    );
    draw_ui_text_ex(
        &format!(
            "Depth {} vU  Heat {} dK",
            selected.surface_volume(),
            heat_dk
        ),
        1024.0,
        244.0,
        style.params(),
    );
    draw_ui_text_ex(
        &format!("Ground {} bp", selected.ground_contamination_bp),
        1024.0,
        262.0,
        style.params(),
    );
    draw_ui_text_ex(&device_readout, 1024.0, 280.0, style.params());
    draw_ui_text_ex(
        &format!(
            "Queue {} / {} credits",
            ctx.session.simulation.devices.queued.len(),
            ctx.session.simulation.devices.reserved_budget
        ),
        1024.0,
        298.0,
        style.params(),
    );
    let verification_status = ctx
        .verification_label
        .map(|_| {
            format!(
                "VERIFY {} B{:+} I{} D{} R{} P{}",
                if ctx.session.simulation.mass_balance_error() == 0 {
                    "PASS"
                } else {
                    "FAIL"
                },
                ctx.session.simulation.mass_balance_error(),
                ctx.session.simulation.ledger.injected,
                ctx.session.simulation.ledger.drained,
                ctx.session.simulation.ledger.reacted,
                ctx.session.simulation.ledger.products,
            )
        })
        .unwrap_or_else(|| "Ledger inactive".into());
    let placement_status = if ctx.placement_valid {
        "READY"
    } else {
        "BLOCKED"
    };
    let build_readout = if ctx.verification_label.is_some() {
        verification_status
    } else {
        format!(
            "BUILD {} {} {}c {}°",
            placement_status,
            ctx.placement_reason,
            ctx.placement_device.cost(),
            ctx.placement_rotation * 90
        )
    };
    draw_ui_text_ex(&build_readout, 1024.0, 316.0, style.params());
    draw_ui_text_ex(
        "BUILD PALETTE // click to queue",
        1024.0,
        330.0,
        TextStyle::new(12.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
    );
    for (index, device) in crate::devices::DeviceId::ALL.into_iter().enumerate() {
        let column = index % 2;
        let row = index / 2;
        let x = 1018.0 + column as f32 * 114.0;
        let y = 344.0 + row as f32 * 26.0;
        draw_rectangle(x, y, 108.0, 22.0, Color::new(0.10, 0.13, 0.17, 0.95));
        draw_rectangle_lines(x, y, 108.0, 22.0, 1.0, Color::new(0.28, 0.38, 0.44, 0.9));
        draw_ui_text_ex(
            device.name(),
            x + 7.0,
            y + 15.0,
            TextStyle::new(11.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
        );
    }
    for (x, label) in [(1018.0, "ROTATE"), (1094.0, "COMMIT"), (1172.0, "CANCEL")] {
        draw_rectangle(x, 468.0, 72.0, 22.0, Color::new(0.12, 0.16, 0.2, 0.98));
        draw_rectangle_lines(x, 468.0, 72.0, 22.0, 1.0, Color::new(0.45, 0.55, 0.62, 0.9));
        draw_ui_text_ex(
            label,
            x + 7.0,
            483.0,
            TextStyle::new(10.0, Color::new(0.82, 0.87, 0.9, 1.0)).params(),
        );
    }
    for (x, label) in [
        (1018.0, "PAUSE"),
        (1078.0, "1X"),
        (1138.0, "2X"),
        (1198.0, "4X"),
    ] {
        draw_rectangle(x, 496.0, 52.0, 22.0, Color::new(0.12, 0.16, 0.2, 0.98));
        draw_rectangle_lines(x, 496.0, 52.0, 22.0, 1.0, Color::new(0.45, 0.55, 0.62, 0.9));
        draw_ui_text_ex(
            label,
            x + 8.0,
            511.0,
            TextStyle::new(10.0, Color::new(0.82, 0.87, 0.9, 1.0)).params(),
        );
    }
    if ctx.verification_label.is_some() {
        for (x, label) in [(1018.0, "RESET"), (1094.0, "STEP"), (1172.0, "RETURN")] {
            draw_rectangle(x, 532.0, 72.0, 28.0, Color::new(0.1, 0.22, 0.27, 0.98));
            draw_rectangle_lines(
                x,
                532.0,
                72.0,
                28.0,
                1.0,
                Color::new(0.35, 0.74, 0.78, 0.95),
            );
            draw_ui_text_ex(
                label,
                x + 8.0,
                550.0,
                TextStyle::new(10.0, Color::new(0.85, 0.94, 0.9, 1.0)).params(),
            );
        }
    }
    if ctx.verification_label.is_none() {
        draw_ui_text_ex(
            "FIELD TOOLS",
            1018.0,
            528.0,
            TextStyle::new(10.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
        );
        for (x, label) in [(1018.0, "INSPECT"), (1138.0, "EXCAVATE")] {
            draw_rectangle(x, 532.0, 108.0, 28.0, Color::new(0.1, 0.22, 0.27, 0.98));
            draw_rectangle_lines(
                x,
                532.0,
                108.0,
                28.0,
                1.0,
                Color::new(0.35, 0.74, 0.78, 0.95),
            );
            draw_ui_text_ex(
                label,
                x + 14.0,
                550.0,
                TextStyle::new(9.0, Color::new(0.85, 0.94, 0.9, 1.0)).params(),
            );
        }
        for (x, label) in [(1018.0, "CLOSED"), (1094.0, "50%"), (1172.0, "OPEN")] {
            draw_rectangle(x, 566.0, 72.0, 28.0, Color::new(0.1, 0.22, 0.27, 0.98));
            draw_rectangle_lines(
                x,
                566.0,
                72.0,
                28.0,
                1.0,
                Color::new(0.35, 0.74, 0.78, 0.95),
            );
            draw_ui_text_ex(
                label,
                x + 6.0,
                584.0,
                TextStyle::new(9.0, Color::new(0.85, 0.94, 0.9, 1.0)).params(),
            );
        }
    }
    if matches!(
        ctx.session.mission.phase,
        MissionPhase::Success | MissionPhase::Failure | MissionPhase::Debrief
    ) {
        draw_terminal_panel(ctx.session);
    }
    if ctx.session.mission.phase == MissionPhase::Briefing {
        draw_mission_briefing(ctx.session);
    }
    if let Some(tutorial) = ctx.session.mission.tutorial.as_ref() {
        if ctx.session.mission.phase == MissionPhase::Active && !tutorial.is_complete() {
            crate::ui_tutorial::draw_tutorial_prompt(tutorial);
        }
    }
    if ctx.verification_label.is_none()
        && ctx.session.mission.phase == MissionPhase::Active
        && ctx.session.mission.id != crate::mission::MissionId::L01FirstFlow
    {
        draw_stage_guide(ctx.session.mission.id);
    }
    if ctx.pause_menu {
        draw_pause_menu();
    }
}

fn time_name(time: TimeControl) -> &'static str {
    match time {
        TimeControl::Paused => "PAUSED",
        TimeControl::OneX => "1X",
        TimeControl::TwoX => "2X",
        TimeControl::FourX => "4X",
    }
}
fn phase_name(phase: MissionPhase) -> &'static str {
    match phase {
        MissionPhase::Briefing => "BRIEFING",
        MissionPhase::Active => "ACTIVE",
        MissionPhase::Success => "SUCCESS",
        MissionPhase::Failure => "FAILURE",
        MissionPhase::Debrief => "DEBRIEF",
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
fn objective_target(id: crate::mission::MissionId) -> u32 {
    match id {
        crate::mission::MissionId::L01FirstFlow | crate::mission::MissionId::L02HoldingLine => {
            6_000
        }
        crate::mission::MissionId::L03Firebreak => 3_000,
    }
}

fn stability_target(id: crate::mission::MissionId) -> u32 {
    match id {
        crate::mission::MissionId::L01FirstFlow => 100,
        crate::mission::MissionId::L02HoldingLine | crate::mission::MissionId::L03Firebreak => 150,
    }
}

fn draw_terminal_panel(session: &GameSession) {
    let success = session.mission.phase == MissionPhase::Success;
    draw_rectangle(
        300.0,
        230.0,
        680.0,
        190.0,
        Color::new(0.035, 0.05, 0.08, 0.98),
    );
    draw_rectangle_lines(
        300.0,
        230.0,
        680.0,
        190.0,
        2.0,
        if success {
            Color::new(0.35, 0.92, 0.72, 1.0)
        } else {
            Color::new(0.95, 0.35, 0.3, 1.0)
        },
    );
    let title = if success {
        "MISSION SUCCESS // STABLE RESULT"
    } else {
        "MISSION FAILURE // RECOVERY AVAILABLE"
    };
    draw_ui_text_ex(
        title,
        336.0,
        274.0,
        TextStyle::new(
            24.0,
            if success {
                Color::new(0.35, 0.92, 0.72, 1.0)
            } else {
                Color::new(0.95, 0.45, 0.38, 1.0)
            },
        )
        .params(),
    );
    draw_ui_text_ex(
        session.mission.id.name(),
        336.0,
        310.0,
        TextStyle::new(16.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
    );
    draw_ui_text_ex(
        &format!(
            "Objective progress: {} vU",
            session.mission.objective_progress
        ),
        336.0,
        334.0,
        TextStyle::new(16.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
    );
    draw_ui_text_ex(
        session
            .mission
            .failure_reason
            .as_deref()
            .unwrap_or("All mandatory conditions held through the stability window."),
        336.0,
        358.0,
        TextStyle::new(15.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
    );
    draw_ui_text_ex(
        if success {
            "Campaign progress updated"
        } else {
            "Recovery is available from the mission menu"
        },
        336.0,
        396.0,
        TextStyle::new(15.0, Color::new(0.95, 0.8, 0.35, 1.0)).params(),
    );
}

fn draw_mission_briefing(session: &GameSession) {
    let (title, lesson, objective) = match session.mission.id {
        crate::mission::MissionId::L01FirstFlow => (
            "L01 // FIRST FLOW",
            "Learn the survey basics: inspect grade, shape terrain, place a channel, then control the basin.",
            "Guide 6,000 vU of meltwater into the safe basin and hold it stable.",
        ),
        crate::mission::MissionId::L02HoldingLine => (
            "L02 // THE HOLDING LINE",
            "Plan for a scheduled surge. Use storage, spillways, and optional sensing to keep the camp dry.",
            "Hold 6,000 vU safely while the meltwater source rises and falls.",
        ),
        crate::mission::MissionId::L03Firebreak => (
            "L03 // FIREBREAK PROTOCOL",
            "Combine routing with reaction engineering: water and lava form rock and steam.",
            "Build a 3,000 vU firebreak before lava can reach the ancient foundation.",
        ),
    };
    draw_rectangle(
        300.0,
        214.0,
        680.0,
        340.0,
        Color::new(0.03, 0.05, 0.08, 0.98),
    );
    draw_rectangle_lines(
        300.0,
        214.0,
        680.0,
        340.0,
        2.0,
        Color::new(0.35, 0.74, 0.78, 0.95),
    );
    draw_ui_text_ex(
        title,
        338.0,
        270.0,
        TextStyle::new(28.0, Color::new(0.94, 0.84, 0.48, 1.0)).params(),
    );
    draw_ui_text_ex(
        "FIELD BRIEFING",
        338.0,
        300.0,
        TextStyle::new(14.0, Color::new(0.52, 0.74, 0.78, 1.0)).params(),
    );
    draw_ui_text_ex(
        lesson,
        338.0,
        346.0,
        TextStyle::new(16.0, Color::new(0.78, 0.84, 0.88, 1.0)).params(),
    );
    draw_ui_text_ex(
        objective,
        338.0,
        388.0,
        TextStyle::new(16.0, Color::new(0.78, 0.84, 0.88, 1.0)).params(),
    );
    draw_rectangle(490.0, 492.0, 300.0, 50.0, Color::new(0.1, 0.22, 0.27, 1.0));
    draw_rectangle_lines(
        490.0,
        492.0,
        300.0,
        50.0,
        1.5,
        Color::new(0.35, 0.82, 0.76, 1.0),
    );
    draw_ui_text_ex(
        "BEGIN OPERATION",
        550.0,
        523.0,
        TextStyle::new(18.0, Color::new(0.85, 0.94, 0.9, 1.0)).params(),
    );
    draw_ui_text_ex(
        "Tap BEGIN OPERATION",
        530.0,
        574.0,
        TextStyle::new(14.0, Color::new(0.56, 0.66, 0.72, 1.0)).params(),
    );
}

fn draw_stage_guide(id: crate::mission::MissionId) {
    let (heading, guidance) = match id {
        crate::mission::MissionId::L02HoldingLine => (
            "FIELD GUIDE // HOLDING LINE",
            "Watch the alert level. The source surges at tick 900; reservoirs retain water and spillways shed excess.",
        ),
        crate::mission::MissionId::L03Firebreak => (
            "FIELD GUIDE // FIREBREAK",
            "Bring water to lava deliberately: their reaction creates the rock barrier. Keep lava away from the foundation.",
        ),
        crate::mission::MissionId::L01FirstFlow => return,
    };
    draw_rectangle(
        24.0,
        104.0,
        620.0,
        76.0,
        Color::new(0.035, 0.055, 0.08, 0.96),
    );
    draw_rectangle_lines(
        24.0,
        104.0,
        620.0,
        76.0,
        1.5,
        Color::new(0.52, 0.74, 0.78, 0.9),
    );
    draw_ui_text_ex(
        heading,
        42.0,
        130.0,
        TextStyle::new(13.0, Color::new(0.52, 0.78, 0.8, 1.0)).params(),
    );
    draw_ui_text_ex(
        guidance,
        42.0,
        158.0,
        TextStyle::new(14.0, Color::new(0.84, 0.88, 0.9, 1.0)).params(),
    );
}

fn draw_pause_menu() {
    draw_rectangle(
        420.0,
        178.0,
        440.0,
        250.0,
        Color::new(0.03, 0.045, 0.07, 0.98),
    );
    draw_rectangle_lines(
        420.0,
        178.0,
        440.0,
        250.0,
        2.0,
        Color::new(0.95, 0.8, 0.35, 1.0),
    );
    draw_ui_text_ex(
        "SURVEY PAUSED",
        478.0,
        224.0,
        TextStyle::new(28.0, Color::new(0.95, 0.8, 0.35, 1.0)).params(),
    );
    draw_ui_text_ex(
        "Simulation and commands are frozen",
        478.0,
        258.0,
        TextStyle::new(16.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
    );
    draw_ui_text_ex(
        "Use the time controls to resume survey",
        478.0,
        304.0,
        TextStyle::new(16.0, WHITE).params(),
    );
    draw_ui_text_ex(
        "Mission progress is saved at checkpoints",
        478.0,
        334.0,
        TextStyle::new(16.0, WHITE).params(),
    );
    draw_ui_text_ex(
        "Reset is available from the mission menu",
        478.0,
        364.0,
        TextStyle::new(16.0, WHITE).params(),
    );
}

fn draw_text_right(text: &str, right: f32, y: f32, style: TextStyle) {
    let dims = measure_text(text, None, style.font_size as u16, 1.0);
    draw_ui_text_ex(text, right - dims.width, y, style.params());
}
