//! Title, campaign, and verification-map entry screens.

use crate::{
    devices::DeviceId,
    game::{FrontendMode, Game},
    mission::MissionId,
    state::GameSession,
    ui,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const MENU_X: f32 = 418.0;
const MENU_WIDTH: f32 = 444.0;
const BUTTON_HEIGHT: f32 = 48.0;

impl Game {
    pub(crate) fn update_frontend(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.frontend_mode = FrontendMode::Title;
            return;
        }
        let choice = keyboard_choice(self.frontend_mode);
        let choice = choice.or_else(|| mouse_choice(self.frontend_mode));
        let Some(choice) = choice else { return };

        match (self.frontend_mode, choice) {
            (FrontendMode::Title, 0) => self.start_new_campaign(),
            (FrontendMode::Title, 1) => self.frontend_mode = FrontendMode::CampaignSelect,
            (FrontendMode::Title, 2) => self.frontend_mode = FrontendMode::VerificationSelect,
            (FrontendMode::CampaignSelect, 0..=2) => {
                let id = MissionId::ALL[choice];
                if self.session.campaign.unlocked[id.sequence() - 1] {
                    self.load_mission(id, "selected from campaign board");
                } else {
                    self.notice = format!(
                        "{} is locked — finish earlier campaign work first",
                        id.name()
                    );
                }
            }
            (FrontendMode::VerificationSelect, 0) => {
                self.verification_returns_to_menu = true;
                self.toggle_lab_mode();
            }
            (FrontendMode::VerificationSelect, 1..=10) => {
                self.verification_returns_to_menu = true;
                self.enter_showcase(DeviceId::ALL[choice - 1]);
            }
            _ => {}
        }
    }

    pub(crate) fn draw_frontend(&self) {
        draw_rectangle(
            0.0,
            0.0,
            ui::LOGICAL_WIDTH,
            ui::LOGICAL_HEIGHT,
            Color::new(0.025, 0.04, 0.07, 1.0),
        );
        for ring in [0.0, 1.0, 2.0] {
            draw_circle_lines(
                640.0,
                340.0,
                150.0 + ring * 62.0,
                1.0,
                Color::new(0.14, 0.29, 0.35, 0.35 - ring * 0.08),
            );
        }
        draw_ui_text_ex(
            "PLANETFALL",
            386.0,
            120.0,
            TextStyle::new(54.0, Color::new(0.94, 0.84, 0.48, 1.0)).params(),
        );
        draw_ui_text_ex(
            "ENGINEER",
            507.0,
            178.0,
            TextStyle::new(54.0, Color::new(0.66, 0.84, 0.88, 1.0)).params(),
        );
        draw_ui_text_ex(
            "A deterministic planetary-restoration field manual",
            410.0,
            210.0,
            TextStyle::new(16.0, Color::new(0.55, 0.66, 0.73, 1.0)).params(),
        );

        match self.frontend_mode {
            FrontendMode::Title => self.draw_title_menu(),
            FrontendMode::CampaignSelect => self.draw_campaign_menu(),
            FrontendMode::VerificationSelect => self.draw_verification_menu(),
            FrontendMode::Playing => unreachable!(),
        }
    }

    fn draw_title_menu(&self) {
        draw_ui_text_ex(
            "FIELD COMMAND",
            553.0,
            278.0,
            TextStyle::new(16.0, Color::new(0.8, 0.68, 0.4, 1.0)).params(),
        );
        draw_menu_button(0, "NEW CAMPAIGN", "Begin at L01: First Flow", true);
        draw_menu_button(
            1,
            "CAMPAIGN BOARD",
            "Choose an unlocked restoration stage",
            true,
        );
        draw_menu_button(
            2,
            "VERIFICATION GROUNDS",
            "Test every fluid and device",
            true,
        );
        draw_ui_text_ex(
            "Enter starts a new campaign   C opens the campaign board   V opens verification",
            320.0,
            548.0,
            TextStyle::new(14.0, Color::new(0.54, 0.63, 0.68, 1.0)).params(),
        );
    }

    fn draw_campaign_menu(&self) {
        draw_ui_text_ex(
            "CAMPAIGN BOARD",
            520.0,
            260.0,
            TextStyle::new(24.0, Color::new(0.9, 0.8, 0.48, 1.0)).params(),
        );
        for (index, id) in MissionId::ALL.into_iter().enumerate() {
            let unlocked = self.session.campaign.unlocked[id.sequence() - 1];
            let detail = if unlocked {
                "Ready for deployment"
            } else {
                "Locked — complete the preceding stage"
            };
            draw_menu_button(
                index,
                &format!("L0{} — {}", id.sequence(), id.name()),
                detail,
                unlocked,
            );
        }
        draw_ui_text_ex(
            "Press 1, 2, or 3 to deploy an unlocked stage. Escape returns to the title screen.",
            296.0,
            520.0,
            TextStyle::new(14.0, Color::new(0.54, 0.63, 0.68, 1.0)).params(),
        );
    }

    fn draw_verification_menu(&self) {
        draw_ui_text_ex(
            "VERIFICATION GROUNDS",
            492.0,
            252.0,
            TextStyle::new(24.0, Color::new(0.9, 0.8, 0.48, 1.0)).params(),
        );
        draw_menu_button(
            0,
            "ALL-FLUID LABORATORY",
            "Observe water, lava, steam, and toxic slurry",
            true,
        );
        for (index, device) in DeviceId::ALL.into_iter().enumerate() {
            let column = index % 2;
            let row = index / 2;
            let x = 300.0 + column as f32 * 344.0;
            let y = 366.0 + row as f32 * 48.0;
            draw_button_at(
                x,
                y,
                320.0,
                42.0,
                &format!("{} TEST BAY", device.name().to_uppercase()),
                "Launch",
                true,
            );
        }
        draw_ui_text_ex(
            "Press 1 for the fluid laboratory; 2–0 select device bays; - selects the final bay; Escape returns.",
            282.0,
            652.0,
            TextStyle::new(14.0, Color::new(0.54, 0.63, 0.68, 1.0)).params(),
        );
    }

    fn start_new_campaign(&mut self) {
        self.session = GameSession::new(&self.data.config);
        self.checkpoint_session = None;
        self.saved_campaign_session = None;
        self.verification_mode = None;
        self.verification_returns_to_menu = false;
        self.load_mission(MissionId::L01FirstFlow, "new campaign deployment");
    }
}

fn draw_menu_button(index: usize, title: &str, detail: &str, enabled: bool) {
    draw_button_at(
        MENU_X,
        300.0 + index as f32 * 62.0,
        MENU_WIDTH,
        BUTTON_HEIGHT,
        title,
        detail,
        enabled,
    );
}

fn draw_button_at(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    title: &str,
    detail: &str,
    enabled: bool,
) {
    let fill = if enabled {
        Color::new(0.08, 0.15, 0.21, 0.96)
    } else {
        Color::new(0.08, 0.09, 0.11, 0.96)
    };
    let border = if enabled {
        Color::new(0.31, 0.58, 0.64, 0.95)
    } else {
        Color::new(0.25, 0.28, 0.31, 0.9)
    };
    draw_rectangle(x, y, width, height, fill);
    draw_rectangle_lines(x, y, width, height, 1.0, border);
    let title_color = if enabled {
        Color::new(0.84, 0.9, 0.88, 1.0)
    } else {
        Color::new(0.48, 0.5, 0.5, 1.0)
    };
    draw_ui_text_ex(
        title,
        x + 14.0,
        y + 21.0,
        TextStyle::new(16.0, title_color).params(),
    );
    draw_ui_text_ex(
        detail,
        x + 14.0,
        y + 39.0,
        TextStyle::new(11.0, Color::new(0.54, 0.63, 0.68, 1.0)).params(),
    );
}

fn keyboard_choice(mode: FrontendMode) -> Option<usize> {
    match mode {
        FrontendMode::Title => {
            if is_key_pressed(KeyCode::Enter) {
                Some(0)
            } else if is_key_pressed(KeyCode::C) {
                Some(1)
            } else if is_key_pressed(KeyCode::V) {
                Some(2)
            } else {
                None
            }
        }
        FrontendMode::CampaignSelect => {
            if is_key_pressed(KeyCode::Key1) {
                Some(0)
            } else if is_key_pressed(KeyCode::Key2) {
                Some(1)
            } else if is_key_pressed(KeyCode::Key3) {
                Some(2)
            } else {
                None
            }
        }
        FrontendMode::VerificationSelect => {
            if is_key_pressed(KeyCode::Key1) {
                Some(0)
            } else if is_key_pressed(KeyCode::Key2) {
                Some(1)
            } else if is_key_pressed(KeyCode::Key3) {
                Some(2)
            } else if is_key_pressed(KeyCode::Key4) {
                Some(3)
            } else if is_key_pressed(KeyCode::Key5) {
                Some(4)
            } else if is_key_pressed(KeyCode::Key6) {
                Some(5)
            } else if is_key_pressed(KeyCode::Key7) {
                Some(6)
            } else if is_key_pressed(KeyCode::Key8) {
                Some(7)
            } else if is_key_pressed(KeyCode::Key9) {
                Some(8)
            } else if is_key_pressed(KeyCode::Key0) {
                Some(9)
            } else if is_key_pressed(KeyCode::Minus) {
                Some(10)
            } else {
                None
            }
        }
        FrontendMode::Playing => None,
    }
}

fn mouse_choice(mode: FrontendMode) -> Option<usize> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }
    let (mouse_x, mouse_y) = mouse_position();
    let x = mouse_x / (screen_width() / ui::LOGICAL_WIDTH);
    let y = mouse_y / (screen_height() / ui::LOGICAL_HEIGHT);
    frontend_mouse_choice_at(mode, x, y)
}

pub(crate) fn frontend_mouse_choice_at(mode: FrontendMode, x: f32, y: f32) -> Option<usize> {
    match mode {
        FrontendMode::Title | FrontendMode::CampaignSelect => (MENU_X..=MENU_X + MENU_WIDTH)
            .contains(&x)
            .then_some(((y - 300.0) / 62.0) as usize)
            .filter(|choice| *choice < 3 && (300.0..=300.0 + 3.0 * 62.0).contains(&y)),
        FrontendMode::VerificationSelect => {
            if (MENU_X..=MENU_X + MENU_WIDTH).contains(&x) && (300.0..=348.0).contains(&y) {
                return Some(0);
            }
            if !(300.0..=964.0).contains(&x) || !(366.0..=608.0).contains(&y) {
                return None;
            }
            let column = usize::from(x >= 644.0);
            let row = ((y - 366.0) / 48.0) as usize;
            let choice = row * 2 + column + 1;
            (choice <= 10).then_some(choice)
        }
        FrontendMode::Playing => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_buttons_have_distinct_click_targets() {
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::Title, 640.0, 324.0),
            Some(0)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::Title, 640.0, 386.0),
            Some(1)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::Title, 640.0, 448.0),
            Some(2)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::Title, 100.0, 324.0),
            None
        );
    }

    #[test]
    fn verification_buttons_select_lab_and_each_device_bay() {
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 640.0, 324.0),
            Some(0)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 460.0, 387.0),
            Some(1)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 800.0, 387.0),
            Some(2)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 460.0, 579.0),
            Some(9)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 800.0, 579.0),
            Some(10)
        );
        assert_eq!(
            frontend_mouse_choice_at(FrontendMode::VerificationSelect, 640.0, 620.0),
            None
        );
    }
}
