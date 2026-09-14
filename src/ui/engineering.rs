//! Engineering readout, build palette, time controls, and field tools.

use super::UiContext;
use crate::ui::status::{device_status, mission_primary_status, mission_secondary_status};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_engineering_panel(ctx: &UiContext<'_>) {
    draw_rectangle(
        1010.0,
        104.0,
        238.0,
        584.0,
        Color::new(0.04, 0.06, 0.09, 0.9),
    );
    draw_ui_text_ex(
        "ENGINEERING READOUT",
        1024.0,
        130.0,
        TextStyle::new(14.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
    );
    draw_selection_summary(ctx);
    draw_build_summary(ctx);
    draw_palette();
    draw_time_controls();
    if ctx.verification_label.is_some() {
        draw_verification_controls();
    } else {
        draw_field_controls(ctx);
    }
}

fn draw_selection_summary(ctx: &UiContext<'_>) {
    let Some(selected_index) = ctx.session.simulation.index(ctx.session.selected) else {
        return;
    };
    let Some(selected) = ctx.session.simulation.cells.get(selected_index) else {
        return;
    };
    let Some(selected_definition) = ctx.session.simulation.definitions.get(selected_index) else {
        return;
    };
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
        .map(device_status)
        .unwrap_or_else(|| "Device none".into());
    let deposit_readout = ctx
        .session
        .simulation
        .deposits
        .iter()
        .find(|deposit| deposit.position == ctx.session.selected)
        .map(|deposit| {
            format!(
                "Deposit {} {}",
                deposit.id,
                if deposit.depleted {
                    "DEPLETED".into()
                } else {
                    format!("{} fabU AVAILABLE", deposit.yield_fu)
                }
            )
        })
        .unwrap_or_else(|| "Deposit none".into());
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
        .unwrap_or_else(|| mission_primary_status(ctx.session, ctx.content));
    let objective = if ctx.verification_label.is_some() {
        objective
    } else {
        format!(
            "{}  STAB {}/{}",
            objective,
            ctx.session.mission.stability_ticks,
            super::stability_target(ctx.session.mission.id, ctx.content)
        )
    };
    draw_ui_text_ex(&objective, 1024.0, 172.0, style.params());
    if ctx.verification_label.is_none() {
        draw_ui_text_ex(
            &mission_secondary_status(ctx.session, ctx.content),
            1024.0,
            190.0,
            style.params(),
        );
    }
    draw_ui_text_ex(
        &format!("ALERT {}", ctx.session.mission.alert_level.name()),
        1024.0,
        208.0,
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
    draw_ui_text_ex(&deposit_readout, 1024.0, 298.0, style.params());
}

fn draw_build_summary(ctx: &UiContext<'_>) {
    let style = TextStyle::new(13.0, Color::new(0.7, 0.76, 0.8, 1.0));
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
            "BUILD {} {} {}fab {}° | A{} R{} P{} D{} B{}",
            placement_status,
            ctx.placement_reason,
            ctx.placement_device.cost(),
            ctx.placement_rotation * 90,
            ctx.session.simulation.fabrication.available_fu,
            ctx.session.simulation.fabrication.reserved_fu,
            ctx.session.simulation.power.supply_eu(),
            ctx.session.simulation.power.allocated_demand_eu,
            ctx.session.simulation.power.deficit_eu,
        )
    };
    draw_ui_text_ex(&build_readout, 1024.0, 316.0, style.params());
}

fn draw_palette() {
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
        draw_small_button(x, y, 108.0, 22.0, device.name(), 11.0);
    }
    for (x, label) in [(1018.0, "ROTATE"), (1094.0, "COMMIT"), (1172.0, "CANCEL")] {
        draw_small_button(x, 468.0, 72.0, 22.0, label, 10.0);
    }
}

fn draw_time_controls() {
    for (x, label) in [
        (1018.0, "PAUSE"),
        (1078.0, "1X"),
        (1138.0, "2X"),
        (1198.0, "4X"),
    ] {
        draw_small_button(x, 496.0, 52.0, 22.0, label, 10.0);
    }
}

fn draw_verification_controls() {
    for (x, label) in [(1018.0, "RESET"), (1094.0, "STEP"), (1172.0, "RETURN")] {
        draw_small_button(x, 532.0, 72.0, 28.0, label, 10.0);
    }
}

fn draw_field_controls(ctx: &UiContext<'_>) {
    draw_ui_text_ex(
        "FIELD TOOLS",
        1018.0,
        528.0,
        TextStyle::new(10.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
    );
    for (x, label) in [(1018.0, "INSPECT"), (1138.0, "EXCAVATE")] {
        draw_wide_button(x, 532.0, label);
    }
    for (x, label) in [(1018.0, "RAISE"), (1138.0, "SEAL")] {
        draw_wide_button(x, 566.0, label);
    }
    let reservoir_selected = ctx.session.simulation.devices.devices.iter().any(|device| {
        device.anchor == ctx.session.selected
            && device.device == crate::devices::DeviceId::Reservoir
    });
    let flow_labels = if reservoir_selected {
        ["DRAIN", "25%", "50%", "HOLD"]
    } else {
        ["STOP", "25%", "50%", "FULL"]
    };
    for (index, label) in flow_labels.into_iter().enumerate() {
        draw_small_button(1018.0 + index as f32 * 58.0, 600.0, 54.0, 28.0, label, 8.0);
    }
    for (x, label) in [(1018.0, "RECOVER"), (1138.0, "POWER")] {
        draw_wide_button(x, 634.0, label);
    }
}

fn draw_small_button(x: f32, y: f32, width: f32, height: f32, label: &str, font_size: f32) {
    draw_rectangle(x, y, width, height, Color::new(0.12, 0.16, 0.2, 0.98));
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.45, 0.55, 0.62, 0.9));
    draw_ui_text_ex(
        label,
        x + 7.0,
        y + height - 7.0,
        TextStyle::new(font_size, Color::new(0.82, 0.87, 0.9, 1.0)).params(),
    );
}

fn draw_wide_button(x: f32, y: f32, label: &str) {
    draw_rectangle(x, y, 108.0, 28.0, Color::new(0.1, 0.22, 0.27, 0.98));
    draw_rectangle_lines(x, y, 108.0, 28.0, 1.0, Color::new(0.35, 0.74, 0.78, 0.95));
    draw_ui_text_ex(
        label,
        x + 14.0,
        y + 18.0,
        TextStyle::new(9.0, Color::new(0.85, 0.94, 0.9, 1.0)).params(),
    );
}
