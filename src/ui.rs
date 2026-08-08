//! Screen-space engineering HUD restored after the 3D world pass.

use crate::{mission::MissionPhase, simulation::FluidId, state::{GameSession, TimeControl}};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

pub struct UiContext<'a> { pub session: &'a GameSession, pub camera_yaw: u8, pub camera_zoom: f32, pub notice: &'a str, pub loaded_assets: usize, pub verification_label: Option<&'a str> }

pub fn draw_hud(ctx: UiContext<'_>) {
    draw_rectangle(0.0, 0.0, LOGICAL_WIDTH, 82.0, Color::new(0.035, 0.05, 0.08, 0.96));
    draw_ui_text_ex("PLANETFALL ENGINEER", 28.0, 39.0, TextStyle::new(28.0, Color::new(0.92, 0.86, 0.68, 1.0)).params());
    let heading = ctx.verification_label.map(str::to_owned).unwrap_or_else(|| format!("L0{} // {} // {}", ctx.session.mission.id.sequence(), ctx.session.mission.id.name().to_uppercase(), phase_name(ctx.session.mission.phase)));
    draw_ui_text_ex(&heading, 30.0, 64.0, TextStyle::new(13.0, Color::new(0.48, 0.62, 0.72, 1.0)).params());
    draw_text_right(&format!("TICK {:04}   HASH {:016X}", ctx.session.tick, ctx.session.state_hash()), 1248.0, 38.0, TextStyle::new(16.0, WHITE));
    draw_text_right(&format!("TIME {}   YAW {}   ZOOM {}", time_name(ctx.session.time_control), u32::from(ctx.camera_yaw) * 90, ctx.camera_zoom as u32), 1248.0, 62.0, TextStyle::new(14.0, Color::new(0.65, 0.75, 0.82, 1.0)));
    draw_rectangle(24.0, 604.0, 1232.0, 88.0, Color::new(0.04, 0.06, 0.09, 0.96));
    draw_ui_text_ex(&format!("SURVEY CURSOR  {}, {}", ctx.session.selected.x, ctx.session.selected.y), 44.0, 634.0, TextStyle::new(18.0, Color::new(0.95, 0.8, 0.35, 1.0)).params());
    draw_ui_text_ex(ctx.notice, 44.0, 660.0, TextStyle::new(15.0, Color::new(0.7, 0.76, 0.8, 1.0)).params());
    draw_ui_text_ex("Click select   WASD pan   Q/E rotate   +/- zoom   Arrows survey   N next level   F1 lab   F2 showcase/return   V next device   Space pause   1/2/4 speed   B/P/O/F queue   Enter commit   Backspace cancel   J/K/H gate   F5/F6/F9/F12 save/checkpoint/load/reset", 44.0, 682.0, TextStyle::new(13.0, Color::new(0.5, 0.58, 0.64, 1.0)).params());
    draw_rectangle(1010.0, 104.0, 238.0, 214.0, Color::new(0.04, 0.06, 0.09, 0.9));
    draw_ui_text_ex("ENGINEERING READOUT", 1024.0, 130.0, TextStyle::new(14.0, Color::new(0.8, 0.68, 0.4, 1.0)).params());
    let selected = &ctx.session.simulation.cells[ctx.session.simulation.index(ctx.session.selected).unwrap()];
    let device_readout = ctx.session.simulation.devices.devices.iter().find(|device| device.anchor == ctx.session.selected).map(|device| format!("Device {} {}% {}", device.device.name(), device.setting_bp / 100, if device.active { "ACTIVE" } else { "IDLE" })).unwrap_or_else(|| "Device none".into());
    let tutorial = ctx.session.mission.tutorial.as_ref().map(|tutorial| tutorial.current_step_id.strip_prefix("tutorial_l01_").unwrap_or(&tutorial.current_step_id)).unwrap_or("none");
    let style = TextStyle::new(13.0, Color::new(0.7, 0.76, 0.8, 1.0));
    draw_ui_text_ex(&format!("Map {}×{}", ctx.session.simulation.width, ctx.session.simulation.height), 1024.0, 154.0, style.params());
    let objective = ctx.verification_label.map(|label| format!("{} tick {}", label, ctx.session.simulation.tick)).unwrap_or_else(|| format!("Objective {} / {} vU", ctx.session.mission.objective_progress, objective_target(ctx.session.mission.id)));
    draw_ui_text_ex(&objective, 1024.0, 172.0, style.params());
    draw_ui_text_ex(&format!("Tutorial {}", tutorial), 1024.0, 190.0, style.params());
    draw_ui_text_ex(&format!("Assets {}   Fluids {}", ctx.loaded_assets, FluidId::ALL.len()), 1024.0, 208.0, style.params());
    draw_ui_text_ex(&format!("Surface {} vU", selected.surface_volume()), 1024.0, 226.0, style.params());
    draw_ui_text_ex(&format!("Ground {} bp", selected.ground_contamination_bp), 1024.0, 244.0, style.params());
    draw_ui_text_ex(&device_readout, 1024.0, 262.0, style.params());
    draw_ui_text_ex(&format!("Queue {} / {} credits", ctx.session.simulation.devices.queued.len(), ctx.session.simulation.devices.reserved_budget), 1024.0, 280.0, style.params());
    if matches!(ctx.session.mission.phase, MissionPhase::Success | MissionPhase::Failure | MissionPhase::Debrief) { draw_terminal_panel(ctx.session); }
}

fn time_name(time: TimeControl) -> &'static str { match time { TimeControl::Paused => "PAUSED", TimeControl::OneX => "1X", TimeControl::TwoX => "2X", TimeControl::FourX => "4X" } }
fn phase_name(phase: MissionPhase) -> &'static str { match phase { MissionPhase::Briefing => "BRIEFING", MissionPhase::Active => "ACTIVE", MissionPhase::Success => "SUCCESS", MissionPhase::Failure => "FAILURE", MissionPhase::Debrief => "DEBRIEF" } }
fn objective_target(id: crate::mission::MissionId) -> u32 { match id { crate::mission::MissionId::L01FirstFlow | crate::mission::MissionId::L02HoldingLine => 6_000, crate::mission::MissionId::L03Firebreak => 3_000 } }

fn draw_terminal_panel(session: &GameSession) {
    let success = session.mission.phase == MissionPhase::Success;
    draw_rectangle(300.0, 230.0, 680.0, 190.0, Color::new(0.035, 0.05, 0.08, 0.98));
    draw_rectangle_lines(300.0, 230.0, 680.0, 190.0, 2.0, if success { Color::new(0.35, 0.92, 0.72, 1.0) } else { Color::new(0.95, 0.35, 0.3, 1.0) });
    let title = if success { "MISSION SUCCESS // STABLE RESULT" } else { "MISSION FAILURE // RECOVERY AVAILABLE" };
    draw_ui_text_ex(title, 336.0, 274.0, TextStyle::new(24.0, if success { Color::new(0.35, 0.92, 0.72, 1.0) } else { Color::new(0.95, 0.45, 0.38, 1.0) }).params());
    draw_ui_text_ex(&format!("{}\nObjective progress: {} vU\n{}", session.mission.id.name(), session.mission.objective_progress, session.mission.failure_reason.as_deref().unwrap_or("All mandatory conditions held through the stability window.")), 336.0, 310.0, TextStyle::new(16.0, Color::new(0.76, 0.82, 0.86, 1.0)).params());
    draw_ui_text_ex(if success { "N  next unlocked level     F12  restart this mission" } else { "F12  restore checkpoint or restart     F9  load saved session" }, 336.0, 396.0, TextStyle::new(15.0, Color::new(0.95, 0.8, 0.35, 1.0)).params());
}

fn draw_text_right(text: &str, right: f32, y: f32, style: TextStyle) { let dims = measure_text(text, None, style.font_size as u16, 1.0); draw_ui_text_ex(text, right - dims.width, y, style.params()); }
