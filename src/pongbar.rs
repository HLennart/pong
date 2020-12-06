use crate::state::{Player, Position};
use crate::println;
use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH};
use spin::RwLock;

pub struct PongBar {
    pub position: RwLock<Position>,
    pub button_pressed: RwLock<Key>,
}

impl PongBar {
    pub(crate) fn new(player: Player) -> Self {
        match player {
            Player::Player1 => PongBar {
                position: RwLock::new(Position {
                    x: 1,
                    y: BUFFER_HEIGHT as u8 / 2,
                }),
                button_pressed: RwLock::new(Key::None),
            },
            Player::Player2 => PongBar {
                position: RwLock::new(Position {
                    x: BUFFER_WIDTH as u8 - 2,
                    y: BUFFER_HEIGHT as u8 / 2,
                }),
                button_pressed: RwLock::new(Key::None),
            },
        }
    }

    pub(crate) fn move_player(&self) {
        let mut btn = self.button_pressed.write();
        match *btn {
            Key::Up => {
                let mut pos = self.position.write();
                if pos.y -3 > 0 {
                    pos.y -= 1;
                }
                *btn = Key::None;
            }
            Key::Down => {
                // Is this the best way to compare u8 to usize?
                let mut pos = self.position.write();
                if usize::from(pos.y) +3 < BUFFER_HEIGHT - 1 {
                    pos.y += 1;
                }
                *btn = Key::None;
            }
            Key::None => {}
        }
    }

    pub fn print_self(&self) {
        println!("pos: {:#?}", self.position.read());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Key {
    Up,
    Down,
    None,
}
