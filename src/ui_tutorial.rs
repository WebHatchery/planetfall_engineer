//! Touch-first tutorial prompt presentation.

use crate::ui::buttons::draw_button;
use crate::{content::ContentRegistry, mission::TutorialState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(crate) fn draw_tutorial_prompt(tutorial: &TutorialState, content: &ContentRegistry) {
    let (step, instruction, action) = tutorial_prompt(content, tutorial.current_step_id.as_str());
    let total_steps = content
        .tutorials
        .iter()
        .find(|record| record.id == tutorial.tutorial_id)
        .map_or(0, |record| record.steps.len());
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
            "FIELD TUTORIAL // STEP {}/{} // {}",
            tutorial.completed_step_ids.len() + 1,
            total_steps,
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
        Rect::new(24.0, 202.0, 164.0, 28.0),
        "SKIP TUTORIAL",
        11.0,
        ButtonTone::Secondary,
    );
    if matches!(
        tutorial.current_step_id.as_str(),
        "tutorial_l01_welcome" | "tutorial_l01_inspect_grade"
    ) {
        draw_button(
            Rect::new(420.0, 114.0, 104.0, 28.0),
            "DISMISS",
            11.0,
            ButtonTone::Positive,
        );
    }
}

fn tutorial_prompt<'a>(content: &'a ContentRegistry, step: &str) -> (&'a str, &'a str, &'a str) {
    if let Some(prompt) = content.tutorial_prompt(step) {
        return (&prompt.title, &prompt.instruction, &prompt.action);
    }
    let _ = step;
    (
        "TUTORIAL",
        "Follow the current mission instruction.",
        "Mission controls",
    )
}
