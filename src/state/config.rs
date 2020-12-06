use core::sync::atomic::{AtomicU8, Ordering};

use crate::vga_buffer::{Color, ColorCode};

use super::Player;

pub struct GameConfig {
    background: Color,
    colors: (PlayerColor, PlayerColor),
}

impl GameConfig {
    pub fn set_color(&self, c: Color, p: Player) {
        match p {
            Player::Player1 => {
                self.colors.0.store(c as u8, Ordering::Relaxed)
            },
            Player::Player2 => {
                self.colors.1.store(c as u8, Ordering::Relaxed)
            }
        }
    }

    pub fn player_color(&self, p: Player) -> Color {
        match p {
            Player::Player1 => {
                self.colors.0.load(Ordering::Relaxed).into()
            },
            Player::Player2 => {
                self.colors.1.load(Ordering::Relaxed).into()
            }
        }
    }

    pub fn player_color_code(&self, p: Player) -> ColorCode {
        let color = self.player_color(p);
        ColorCode::new(color, self.background)
    }

    pub fn reset_colors(&self) {
        self.colors.0.store(14, Ordering::Relaxed);
        self.colors.1.store(14, Ordering::Relaxed);
    }
}

type PlayerColor = AtomicU8;

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            background: Color::Black,
            colors: (AtomicU8::new(14), AtomicU8::new(14)),
        }
    }
}