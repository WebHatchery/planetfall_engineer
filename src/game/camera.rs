//! Orthographic survey camera state and keyboard motion.

use crate::ui;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FoundationCamera {
    pub(crate) target: Vec2,
    pub(crate) yaw: u8,
    pub(crate) zoom: f32,
}

impl FoundationCamera {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            target: vec2(width as f32 / 2.0, height as f32 / 2.0),
            yaw: 0,
            zoom: (width.max(height) as f32 * 0.52).clamp(18.0, 28.0),
        }
    }

    pub(crate) fn update(&mut self, dt: f32, width: usize, height: usize) -> bool {
        let before = (self.target, self.yaw, self.zoom);
        let speed = dt * self.zoom * 0.8;
        if is_key_down(KeyCode::A) {
            self.target.x -= speed;
        }
        if is_key_down(KeyCode::D) {
            self.target.x += speed;
        }
        if is_key_down(KeyCode::W) {
            self.target.y -= speed;
        }
        if is_key_down(KeyCode::S) {
            self.target.y += speed;
        }
        self.target.x = self.target.x.clamp(0.0, width as f32);
        self.target.y = self.target.y.clamp(0.0, height as f32);
        if is_key_pressed(KeyCode::Q) {
            self.yaw = (self.yaw + 3) % 4;
        }
        if is_key_pressed(KeyCode::E) {
            self.yaw = (self.yaw + 1) % 4;
        }
        if is_key_pressed(KeyCode::Minus) {
            self.zoom = (self.zoom + 4.0).min(54.0);
        }
        if is_key_pressed(KeyCode::Equal) {
            self.zoom = (self.zoom - 4.0).max(12.0);
        }
        before != (self.target, self.yaw, self.zoom)
    }

    pub(crate) fn camera3d(&self) -> Camera3D {
        let angle = self.yaw as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        let distance = self.zoom * 1.65;
        let target = vec3(self.target.x, 0.0, self.target.y);
        let viewport_scale = (screen_width() / ui::LOGICAL_WIDTH)
            .min(screen_height() / ui::LOGICAL_HEIGHT)
            .max(0.5);
        Camera3D {
            position: target
                + vec3(
                    angle.cos() * distance,
                    distance * 0.82,
                    angle.sin() * distance,
                ),
            target,
            up: vec3(0.0, 1.0, 0.0),
            projection: Projection::Orthographics,
            fovy: self.zoom / viewport_scale,
            aspect: Some(screen_width() / screen_height()),
            ..Default::default()
        }
    }
}
