use macroquad::prelude::*;

pub struct Lobby {
    text_field: String,
    logo: Texture2D,
}

impl Lobby {
    pub fn new(logo: Texture2D) -> Self {
        Self {
            text_field: "".to_owned(),
            logo,
        }
    }

    pub fn run(&mut self) -> Option<String> {
        if is_key_pressed(KeyCode::Key0) {
            self.text_field.push_str("0");
        }
        if is_key_pressed(KeyCode::Key1) {
            self.text_field.push_str("1");
        }
        if is_key_pressed(KeyCode::Key2) {
            self.text_field.push_str("2");
        }
        if is_key_pressed(KeyCode::Key3) {
            self.text_field.push_str("3");
        }
        if is_key_pressed(KeyCode::Key4) {
            self.text_field.push_str("4");
        }
        if is_key_pressed(KeyCode::Key5) {
            self.text_field.push_str("5");
        }
        if is_key_pressed(KeyCode::Key6) {
            self.text_field.push_str("6");
        }
        if is_key_pressed(KeyCode::Key7) {
            self.text_field.push_str("7");
        }
        if is_key_pressed(KeyCode::Key8) {
            self.text_field.push_str("8");
        }
        if is_key_pressed(KeyCode::Key9) {
            self.text_field.push_str("9");
        }
        if is_key_pressed(KeyCode::Backspace) {
            let mut chars = self.text_field.chars();
            chars.next_back();
            self.text_field = chars.as_str().to_owned();
        }

        if self.text_field.len() > 4 {
            self.text_field = self.text_field[0..4].to_owned();
        }

        self.render();

        if is_key_pressed(KeyCode::Enter) && self.text_field.len() == 4 {
            Some(self.text_field.clone())
        } else if is_key_pressed(KeyCode::Enter) && self.text_field.len() == 0 {
            Some("random".to_owned())
        } else {
            None
        }
    }

    fn render(&self) {
        clear_background(BLACK);
        let dest_x = screen_width() / 2.0;
        let dest_y = self.logo.height() * (dest_x / self.logo.width());
        draw_texture_ex(
            self.logo,
            screen_width() / 2. - dest_x / 2.,
            20.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest_x, dest_y)),
                ..Default::default()
            },
        );
        let text_x = screen_width() / 2. + dest_x / 2. - 120.;
        draw_text("DEMO", text_x, dest_y + 30., 50., WHITE);
        draw_text(
            "- enter a lobby ID (4 digits) to play with a friend",
            20.0,
            dest_y + 60.0,
            30.0,
            WHITE,
        );
        draw_text(
            "- leave empty to get matched against a random person",
            20.0,
            dest_y + 90.0,
            30.0,
            WHITE,
        );
        draw_text(
            "- Then, press ENTER to start!",
            20.0,
            dest_y + 120.0,
            30.0,
            WHITE,
        );

        let lobby_code_str = format!("Lobby Code: {}", self.text_field);
        draw_text(&lobby_code_str, 20.0, dest_y + 200.0, 80.0, YELLOW);
    }
}
