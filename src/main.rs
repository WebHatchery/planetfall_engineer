//! Planetfall Engineer prototype wired to macroquad-toolkit.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod data;
mod content;
mod campaign;
mod devices;
mod game;
mod mission;
mod replay;
mod state;
mod simulation;
mod verification;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    capture::capture_window_conf(
        "PLANETFALL_ENGINEER",
        "Planetfall Engineer",
        ui::LOGICAL_WIDTH as i32,
        ui::LOGICAL_HEIGHT as i32,
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when PLANETFALL_ENGINEER_CAPTURE_PATH is set, render
    // deterministic frames, write a PNG, and exit. This is a minimal starter
    // prototype with a single boot state, so the capture photographs
    // whatever the boot flow lands on.
    if let Some(config) = capture::CaptureConfig::from_env("PLANETFALL_ENGINEER") {
        capture::run_capture(&config, |dt| {
            game.update(dt);
            game.draw();
        })
        .await;
        return;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
