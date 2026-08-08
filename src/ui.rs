//! Screen-space engineering HUD restored after the 3D world pass.

use crate::{data::GameData, simulation::FluidId, state::{GameSession, TimeControl}};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

pub struct UiContext<'a> { pub data: &'a GameData, pub session: &'a GameSession, pub camera_yaw: u8, pub camera_zoom: f32, pub notice: &'a str, pub loaded_assets: usize }

pub fn draw_hud(ctx: UiContext<'_>) {
    draw_rectangle(0.0, 0.0, LOGICAL_WIDTH, 82.0, Color::new(0.035, 0.05, 0.08, 0.96));
    draw_ui_text_ex("PLANETFALL ENGINEER", 28.0, 39.0, TextStyle::new(28.0, Color::new(0.92, 0.86, 0.68, 1.0)).params());
    draw_ui_text_ex("PHASE 1 // ORTHOGRAPHIC SURVEY FOUNDATION", 30.0, 64.0, TextStyle::new(13.0, Color::new(0.48, 0.62, 0.72, 1.0)).params());
    draw_text_right(&format!("TICK {:04}   HASH {:016X}", ctx.session.tick, ctx.session.state_hash()), 1248.0, 38.0, TextStyle::new(16.0, WHITE));
    draw_text_right(&format!("TIME {}   YAW {}   ZOOM {}", time_name(ctx.session.time_control), u32::from(ctx.camera_yaw) * 90, ctx.camera_zoom as u32), 1248.0, 62.0, TextStyle::new(14.0, Color::new(0.65, 0.75, 0.82, 1.0)));
    draw_rectangle(24.0, 604.0, 1232.0, 88.0, Color::new(0.04, 0.06, 0.09, 0.96));
    draw_ui_text_ex(&format!("SURVEY CURSOR  {}, {}", ctx.session.selected.x, ctx.session.selected.y), 44.0, 634.0, TextStyle::new(18.0, Color::new(0.95, 0.8, 0.35, 1.0)).params());
    draw_ui_text_ex(ctx.notice, 44.0, 660.0, TextStyle::new(15.0, Color::new(0.7, 0.76, 0.8, 1.0)).params());
    draw_ui_text_ex("Click select   WASD pan   Q/E rotate   +/- zoom   Arrows survey   Space pause   1/2/4 speed   B/P/O build   C remove   F5/F9 save/load", 44.0, 682.0, TextStyle::new(13.0, Color::new(0.5, 0.58, 0.64, 1.0)).params());
    draw_rectangle(1010.0, 104.0, 238.0, 130.0, Color::new(0.04, 0.06, 0.09, 0.9));
    draw_ui_text_ex("ENGINEERING READOUT", 1024.0, 130.0, TextStyle::new(14.0, Color::new(0.8, 0.68, 0.4, 1.0)).params());
    let selected = &ctx.session.simulation.cells[ctx.session.simulation.index(ctx.session.selected).unwrap()];
    draw_ui_text_ex(&format!("Map {}×{}\nAssets loaded {}\nFluids enabled {}\nSurface {} vU\nGround {} bp", ctx.data.config.world_width, ctx.data.config.world_height, ctx.loaded_assets, FluidId::ALL.len(), selected.surface_volume(), selected.ground_contamination_bp), 1024.0, 154.0, TextStyle::new(15.0, Color::new(0.7, 0.76, 0.8, 1.0)).params());
}

fn time_name(time: TimeControl) -> &'static str { match time { TimeControl::Paused => "PAUSED", TimeControl::OneX => "1X", TimeControl::TwoX => "2X", TimeControl::FourX => "4X" } }

fn draw_text_right(text: &str, right: f32, y: f32, style: TextStyle) { let dims = measure_text(text, None, style.font_size as u16, 1.0); draw_ui_text_ex(text, right - dims.width, y, style.params()); }
