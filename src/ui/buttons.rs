//! Shared logical-coordinate button rendering for the engineering interface.

use macroquad::prelude::*;
use macroquad_toolkit::{
    prelude::*,
    ui::{button_rect_enabled_styled_ex_at, virtual_mouse_position, ButtonTrigger},
};

use super::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

pub(crate) fn draw_button(rect: Rect, label: &str, font_size: f32, tone: ButtonTone) {
    let style = ButtonStyle::from_tone(tone);
    let pointer = virtual_mouse_position(LOGICAL_WIDTH, LOGICAL_HEIGHT);
    let _ = button_rect_enabled_styled_ex_at(
        rect,
        label,
        true,
        &style,
        TextStyle::new(font_size, style.text_color),
        ButtonTrigger::Release,
        pointer,
    );
}
