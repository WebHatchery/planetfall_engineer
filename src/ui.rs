//! Screen-space engineering HUD restored after the 3D world pass.

use crate::{
    content::ContentRegistry,
    mission::MissionPhase,
    state::{GameSession, TimeControl},
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

mod engineering;
mod panels;
mod status;
use engineering::draw_engineering_panel;
use panels::{draw_mission_overlays, draw_text_right};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

pub struct UiContext<'a> {
    pub session: &'a GameSession,
    pub content: &'a ContentRegistry,
    pub camera_yaw: u8,
    pub camera_zoom: f32,
    pub notice: &'a str,
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
            let title = ctx
                .content
                .mission(ctx.session.mission.id.content_id())
                .map_or(ctx.session.mission.id.content_id(), |mission| {
                    mission.title.as_str()
                });
            format!(
                "L0{} // {} // {}",
                ctx.session.mission.id.sequence(),
                title.to_uppercase(),
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
    draw_engineering_panel(&ctx);
    draw_mission_overlays(&ctx);
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
fn stability_target(id: crate::mission::MissionId, content: &ContentRegistry) -> u32 {
    content
        .mission(id.content_id())
        .map_or(0, |mission| mission.stability_ticks)
}
