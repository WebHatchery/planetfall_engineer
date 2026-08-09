//! Touch-first tutorial prompt presentation.

use crate::mission::TutorialState;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(crate) fn draw_tutorial_prompt(tutorial: &TutorialState) {
    let (step, instruction, action) = tutorial_prompt(tutorial.current_step_id.as_str());
    draw_rectangle(
        24.0,
        104.0,
        520.0,
        136.0,
        Color::new(0.035, 0.055, 0.08, 0.96),
    );
    draw_rectangle_lines(
        24.0,
        104.0,
        520.0,
        136.0,
        1.5,
        Color::new(0.95, 0.8, 0.35, 0.9),
    );
    draw_ui_text_ex(
        &format!(
            "FIELD TUTORIAL // STEP {}/12 // {}",
            tutorial.completed_step_ids.len() + 1,
            step
        ),
        42.0,
        130.0,
        TextStyle::new(13.0, Color::new(0.95, 0.8, 0.35, 1.0)).params(),
    );
    draw_ui_text_ex(
        instruction,
        42.0,
        158.0,
        TextStyle::new(16.0, Color::new(0.86, 0.9, 0.92, 1.0)).params(),
    );
    draw_ui_text_ex(
        &format!("Required: {action}"),
        42.0,
        184.0,
        TextStyle::new(13.0, Color::new(0.58, 0.72, 0.8, 1.0)).params(),
    );
    draw_button(
        24.0,
        202.0,
        164.0,
        "SKIP TUTORIAL",
        Color::new(0.45, 0.55, 0.62, 0.9),
    );
    if matches!(
        tutorial.current_step_id.as_str(),
        "tutorial_l01_welcome" | "tutorial_l01_inspect_grade"
    ) {
        draw_button(
            420.0,
            114.0,
            104.0,
            "DISMISS",
            Color::new(0.35, 0.82, 0.76, 1.0),
        );
    }
}

fn draw_button(x: f32, y: f32, width: f32, label: &str, border: Color) {
    draw_rectangle(x, y, width, 28.0, Color::new(0.12, 0.16, 0.2, 0.98));
    draw_rectangle_lines(x, y, width, 28.0, 1.0, border);
    draw_ui_text_ex(
        label,
        x + 18.0,
        y + 19.0,
        TextStyle::new(11.0, Color::new(0.82, 0.87, 0.9, 1.0)).params(),
    );
}

fn tutorial_prompt(step: &str) -> (&'static str, &'static str, &'static str) {
    match step {
        "tutorial_l01_welcome" => (
            "WELCOME",
            "Survey the ash basin and dismiss this briefing.",
            "Tap DISMISS",
        ),
        "tutorial_l01_move_camera" => (
            "CAMERA",
            "Adjust the survey view to inspect the ridge.",
            "Drag across the map",
        ),
        "tutorial_l01_move_cursor" => (
            "SURVEY CURSOR",
            "Position the survey cursor on a terrain cell.",
            "Tap a terrain cell on the map",
        ),
        "tutorial_l01_inspect_grade" => (
            "INSPECT",
            "Inspect the selected cell before changing it.",
            "Tap INSPECT in FIELD TOOLS, then DISMISS",
        ),
        "tutorial_l01_pause_plan" => (
            "PAUSE AND PLAN",
            "Pause the simulation before changing the terrain.",
            "Tap PAUSE in the time controls at right",
        ),
        "tutorial_l01_raise_first" => (
            "FIRST BARRIER",
            "Raise the first runoff cut at (8, 10) to block the fissure.",
            "Tap cell (8, 10), then tap RAISE",
        ),
        "tutorial_l01_select_second" => (
            "SECOND CUT",
            "Move east along the stream and select runoff cut (14, 10).",
            "Tap cell (14, 10) on the map",
        ),
        "tutorial_l01_raise_second" => (
            "SECOND BARRIER",
            "Raise the selected cut so water stays in the eastbound course.",
            "Tap RAISE in FIELD TOOLS at right",
        ),
        "tutorial_l01_select_third" => (
            "THIRD CUT",
            "Select the final runoff cut at (20, 10), near the dam.",
            "Tap cell (20, 10) on the map",
        ),
        "tutorial_l01_raise_third" => (
            "THIRD BARRIER",
            "Raise the final cut to complete the route to the dam.",
            "Tap RAISE in FIELD TOOLS at right",
        ),
        "tutorial_l01_run_and_observe" => (
            "OBSERVE",
            "Run time and watch water cross the map toward the dam.",
            "Tap 1X in the time controls at right",
        ),
        "tutorial_l01_stabilize" => (
            "STABILIZE",
            "Keep time running while the far-side dam fills and verifies.",
            "Tap 1X and watch the DAM objective readout",
        ),
        _ => (
            "TUTORIAL",
            "Follow the current mission instruction.",
            "Mission controls",
        ),
    }
}
