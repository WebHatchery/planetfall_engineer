//! Mission, briefing, guide, pause, and terminal panels.

use super::buttons::draw_button;
use crate::{content::ContentRegistry, mission::MissionPhase, state::GameSession};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_mission_overlays(ctx: &super::UiContext<'_>) {
    if matches!(
        ctx.session.mission.phase,
        MissionPhase::Success | MissionPhase::Failure | MissionPhase::Debrief
    ) {
        draw_terminal_panel(ctx.session, ctx.content);
    }
    if ctx.session.mission.phase == MissionPhase::Briefing {
        draw_mission_briefing(ctx.session, ctx.content);
    }
    if let Some(tutorial) = ctx.session.mission.tutorial.as_ref() {
        if ctx.session.mission.phase == MissionPhase::Active && !tutorial.is_complete() {
            crate::ui_tutorial::draw_tutorial_prompt(tutorial, ctx.content);
        }
    }
    if ctx.verification_label.is_none()
        && ctx.session.mission.phase == MissionPhase::Active
        && ctx.session.mission.id != crate::mission::MissionId::L01FirstFlow
    {
        draw_stage_guide(ctx.session.mission.id, ctx.content);
    }
    if ctx.pause_menu {
        draw_pause_menu();
    }
}

pub(super) fn draw_terminal_panel(session: &GameSession, content: &ContentRegistry) {
    let success = session.mission.phase == MissionPhase::Success;
    draw_rectangle(
        300.0,
        230.0,
        680.0,
        210.0,
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
        content
            .mission(session.mission.id.content_id())
            .map_or(session.mission.id.content_id(), |mission| {
                mission.title.as_str()
            }),
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
    let labels = if success {
        ("NEXT MISSION", "CAMPAIGN BOARD")
    } else {
        ("RETRY CHECKPOINT", "RESTART MISSION")
    };
    for (x, label) in [(336.0, labels.0), (660.0, labels.1)] {
        draw_rectangle(x, 374.0, 284.0, 40.0, Color::new(0.10, 0.22, 0.27, 0.98));
        draw_rectangle_lines(
            x,
            374.0,
            284.0,
            40.0,
            1.5,
            if success {
                Color::new(0.35, 0.92, 0.72, 1.0)
            } else {
                Color::new(0.95, 0.45, 0.38, 1.0)
            },
        );
        draw_ui_text_ex(
            label,
            x + 24.0,
            400.0,
            TextStyle::new(14.0, Color::new(0.88, 0.93, 0.9, 1.0)).params(),
        );
    }
}

pub(super) fn draw_mission_briefing(session: &GameSession, content: &ContentRegistry) {
    let Some(mission) = content.mission(session.mission.id.content_id()) else {
        return;
    };
    let title = format!(
        "L0{} // {}",
        session.mission.id.sequence(),
        mission.title.to_uppercase()
    );
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
        &title,
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
        &mission.briefing,
        338.0,
        346.0,
        TextStyle::new(16.0, Color::new(0.78, 0.84, 0.88, 1.0)).params(),
    );
    draw_ui_text_ex(
        &mission.objective,
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

pub(super) fn draw_stage_guide(id: crate::mission::MissionId, content: &ContentRegistry) {
    let Some(mission) = content.mission(id.content_id()) else {
        return;
    };
    if id == crate::mission::MissionId::L01FirstFlow {
        return;
    }
    let heading = format!("FIELD GUIDE // {}", mission.title.to_uppercase());
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
        &heading,
        42.0,
        130.0,
        TextStyle::new(13.0, Color::new(0.52, 0.78, 0.8, 1.0)).params(),
    );
    draw_ui_text_ex(
        &mission.field_guide,
        42.0,
        158.0,
        TextStyle::new(14.0, Color::new(0.84, 0.88, 0.9, 1.0)).params(),
    );
}

pub(super) fn draw_pause_menu() {
    draw_rectangle(
        400.0,
        150.0,
        480.0,
        340.0,
        Color::new(0.03, 0.045, 0.07, 0.98),
    );
    draw_rectangle_lines(
        400.0,
        150.0,
        480.0,
        340.0,
        2.0,
        Color::new(0.95, 0.8, 0.35, 1.0),
    );
    draw_ui_text_ex(
        "SURVEY PAUSED",
        458.0,
        196.0,
        TextStyle::new(28.0, Color::new(0.95, 0.8, 0.35, 1.0)).params(),
    );
    draw_ui_text_ex(
        "Simulation and commands are frozen",
        458.0,
        224.0,
        TextStyle::new(16.0, Color::new(0.76, 0.82, 0.86, 1.0)).params(),
    );
    draw_ui_text_ex(
        "TIME CONTROL",
        430.0,
        238.0,
        TextStyle::new(16.0, WHITE).params(),
    );
    for (x, label) in [
        (430.0, "RESUME 1X"),
        (550.0, "RESUME 2X"),
        (670.0, "RESUME 4X"),
    ] {
        draw_button(
            Rect::new(x, 242.0, 110.0, 32.0),
            label,
            11.0,
            ButtonTone::Positive,
        );
    }
    for (x, label) in [(430.0, "SAVE"), (555.0, "LOAD"), (680.0, "RESET")] {
        draw_button(
            Rect::new(x, 290.0, 125.0, 32.0),
            label,
            11.0,
            ButtonTone::Positive,
        );
    }
    draw_ui_text_ex(
        "Saving and reset preserve the authoritative field ledger",
        430.0,
        354.0,
        TextStyle::new(14.0, WHITE).params(),
    );
    draw_ui_text_ex(
        "Escape returns to the survey",
        430.0,
        382.0,
        TextStyle::new(14.0, Color::new(0.56, 0.66, 0.72, 1.0)).params(),
    );
}

pub(super) fn draw_text_right(text: &str, right: f32, y: f32, style: TextStyle) {
    let dims = measure_text(text, None, style.font_size as u16, 1.0);
    draw_ui_text_ex(text, right - dims.width, y, style.params());
}
