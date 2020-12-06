use spin::RwLock;
use x86_64::instructions::interrupts;

use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, ScreenChar, WRITER};
use crate::{
    pongball::{PongBall, HIGHEST_SPEED, LOWEST_SPEED},
    pongbar::PongBar,
};
// floorf may not be needed? cas (`as`) may already truncate
// not entirely sure tho, so I'm just not gonna try to break it
use libm::{fabsf, floorf};
use spin::Mutex;



pub struct GameState {
    pub player1: PongBar,
    pub player2: PongBar,
    pub ball: Mutex<PongBall>,
    // The proper way to do this would be 2 Atomics
    // But it seems like i'm too smooth brain to understand memory ordering
    // So in case this becomes relevant again:
    // [read this pls](https://en.cppreference.com/w/cpp/atomic/memory_order)
    pub score: RwLock<(u32, u32)>,
    pub config: GameConfig,
}

use crate::vga_buffer::ColorCode;




impl GameState {
    pub fn new() -> Self {
        render_menu_text();
        Self {
            player1: PongBar::new(Player::Player1),
            player2: PongBar::new(Player::Player2),
            ball: Mutex::new(PongBall::new()),
            score: RwLock::new((0, 0)),
            config: GameConfig::default(),
        }
    }

    pub fn step(&self) {
        self.player1.move_player();
        self.player2.move_player();
        self.eval_collisions();
        self.ball.lock().move_ball();
        
        render_ball(&self.ball.lock());
        render_player(&self.player1.position, &self.config.player_color_code(Player::Player1));
        render_player(&self.player2.position, &self.config.player_color_code(Player::Player2));
    }

    pub fn show_menu(&self) {
        self.player1.move_player();
        self.player2.move_player();

        render_player(&self.player1.position, &self.config.player_color_code(Player::Player1));
        render_player(&self.player2.position, &self.config.player_color_code(Player::Player2));
    }

    pub fn reset(&self) {
        self.reset_players();
        // ball; Note: There is some lock black magic fuckery going on here
        {
            reset_ball(&mut *self.ball.lock());
        }
        // score; Note: there is some lock magic going on here
        {
            let mut score = self.score.write();
            *score = (0, 0);
        }
        self.config.reset_colors();

        
    }

    pub fn reset_players(&self) {
        let height = crate::vga_buffer::BUFFER_HEIGHT;
        let mut mutex_guard = WRITER.lock();

        {
            let mut p1_pos = self.player1.position.write();
            *p1_pos = Position {
                x: 1,
                y: BUFFER_HEIGHT as u8 / 2,
            };
            let mut p1_button_pressed = self.player1.button_pressed.write();
            *p1_button_pressed = crate::pongbar::Key::None;

            // Clear screen
            (0..height).for_each(|row| {
                mutex_guard.write_byte_at_pos(row as usize, 1 as usize, b' ');
            });
        }

        {
            let mut p2_pos = self.player2.position.write();
            *p2_pos = Position {
                x: BUFFER_WIDTH as u8 - 2,
                y: BUFFER_HEIGHT as u8 / 2,
            };
            let mut p2_button_pressed = self.player2.button_pressed.write();
            *p2_button_pressed = crate::pongbar::Key::None;

            // Clear screen
            (0..height).for_each(|row| {
                mutex_guard.write_byte_at_pos(row as usize, BUFFER_WIDTH - 2, b' ');
            });
        }
    }

    pub fn eval_collisions(&self) {
        let mut mutex_guard = self.ball.lock();
        let ball_pos_y = floorf(mutex_guard.position.y) as u8;
        let ball_pos_x = floorf(mutex_guard.position.x) as u8;
        //////////////////////////////////////////////////////
        //                  Player collision                //
        //////////////////////////////////////////////////////
        //////////////////////////////////////////////////////
        //                  Player1                         //
        //////////////////////////////////////////////////////
        if mutex_guard.speed.dx < 0.0 {
            let pos_p1 = self.player1.position.read();
            //let ball_pos_x = floorf(mutex_guard.position.x) as u8;

            if ball_pos_x <= 2 && ball_pos_y >= pos_p1.y - 2 && ball_pos_y <= pos_p1.y + 2 {
                mutex_guard.speed.dx = -mutex_guard.speed.dx;

                // CHECK HOW FAR THE BALL IS FROM THE MIDDLE FROM THE BAR
                // THE MORE IT IS IN THE MIDDLE THE SLOWER IT GETS
                let diff = if ball_pos_y >= pos_p1.y {
                    ball_pos_y - pos_p1.y
                } else {
                    pos_p1.y - ball_pos_y
                };

                match diff {
                    0 => {
                        mutex_guard.speed.dx *= 0.85;
                        if mutex_guard.speed.dx < LOWEST_SPEED {
                            mutex_guard.speed.dx = LOWEST_SPEED;
                        }
                        if mutex_guard.speed.dy > 0.0 {
                            mutex_guard.speed.dy *= 0.9;
                            if fabsf(mutex_guard.speed.dy) < LOWEST_SPEED {
                                if mutex_guard.speed.dy < 0.0 {
                                    mutex_guard.speed.dy = -LOWEST_SPEED;
                                } else {
                                    mutex_guard.speed.dy = LOWEST_SPEED;
                                }
                            }
                        }
                    }
                    1 => {
                        mutex_guard.speed.dx *= 1.15;
                        if mutex_guard.speed.dx > HIGHEST_SPEED {
                            mutex_guard.speed.dx = HIGHEST_SPEED;
                        }
                    }
                    2 => {
                        mutex_guard.speed.dy *= 1.35;
                        if fabsf(mutex_guard.speed.dy) > HIGHEST_SPEED {
                            if mutex_guard.speed.dy < 0.0 {
                                mutex_guard.speed.dy = -HIGHEST_SPEED;
                            } else {
                                mutex_guard.speed.dy = HIGHEST_SPEED;
                            }
                        }
                        mutex_guard.speed.dx *= 1.1;
                    }
                    _ => panic!("`diff` greater than 2!: {}\n pos_1.y: {}", diff, pos_p1.y),
                }
            }
            else {
                //////////////////////////////////////////////////////
                //                  Collision left wall             //
                //////////////////////////////////////////////////////
                core::mem::drop(pos_p1);
                if ball_pos_x + floorf(mutex_guard.speed.dx) as u8 <= 0 {
                    // We can easily ignore all interrupts while resetting
                    interrupts::without_interrupts(|| {
                        self.reset_players();
                        reset_ball(&mut mutex_guard);

                        let mut score = self.score.write();
                        score.1 += 1;
                        let score_cached = score.clone();
                        // drop by hand to minimize time where lock is locked
                        // -> because by now im paranoid
                        core::mem::drop(score);
                        render_score(score_cached);
                    });
                }
            }
        }
        //////////////////////////////////////////////////////
        //                  Player2                         //
        //////////////////////////////////////////////////////
        else {
            let pos_p2 = self.player2.position.read();
            let ball_pos_x = floorf(mutex_guard.position.x) as u8;

            if ball_pos_x + 1 >= pos_p2.x
                && ball_pos_y >= pos_p2.y - 2
                && ball_pos_y <= pos_p2.y + 2
            {
                mutex_guard.speed.dx = -mutex_guard.speed.dx;

                let diff = if ball_pos_y >= pos_p2.y {
                    ball_pos_y - pos_p2.y
                } else {
                    pos_p2.y - ball_pos_y
                };

                match diff {
                    0 => {
                        mutex_guard.speed.dx *= 0.85;
                        if mutex_guard.speed.dx > -LOWEST_SPEED {
                            mutex_guard.speed.dx = -LOWEST_SPEED;
                        }
                        if mutex_guard.speed.dy > 0.0 {
                            mutex_guard.speed.dy *= 0.9;
                            if fabsf(mutex_guard.speed.dy) < LOWEST_SPEED {
                                if mutex_guard.speed.dy < 0.0 {
                                    mutex_guard.speed.dy = -LOWEST_SPEED;
                                } else {
                                    mutex_guard.speed.dy = LOWEST_SPEED;
                                }
                            }
                        }
                    }
                    1 => {
                        mutex_guard.speed.dx *= 1.15;
                        if mutex_guard.speed.dx < -HIGHEST_SPEED {
                            mutex_guard.speed.dx = -HIGHEST_SPEED;
                        }
                    }
                    2 => {
                        mutex_guard.speed.dy *= 1.35;
                        if fabsf(mutex_guard.speed.dy) > HIGHEST_SPEED {
                            if mutex_guard.speed.dy < 0.0 {
                                mutex_guard.speed.dy = -HIGHEST_SPEED;
                            } else {
                                mutex_guard.speed.dy = HIGHEST_SPEED;
                            }
                        }
                        mutex_guard.speed.dx *= 1.1;
                    }
                    _ => panic!("`diff` greater than 2!"),
                }
            } 
            else {
                //////////////////////////////////////////////////////
                //                  Collision right wall            //
                //////////////////////////////////////////////////////
                core::mem::drop(pos_p2);
                if ball_pos_x + floorf(mutex_guard.speed.dx) as u8  >= BUFFER_WIDTH as u8 - 1{// - floorf(mutex_guard.speed.dx) as u8 {
                    // We can easily ignore all interrupts while resetting
                    interrupts::without_interrupts(|| {
                        self.reset_players();
                        reset_ball(&mut mutex_guard);
                        let mut score = self.score.write();
                        score.0 += 1;

                        let score_cached = score.clone();
                        // drop by hand to minimize time where lock is locked
                        // -> because by now im paranoid
                        core::mem::drop(score);
                        render_score(score_cached);
                    });
                }
            }
        }

        
        
        //////////////////////////////////////////////////////
        //                  Collision top wall              //
        //////////////////////////////////////////////////////
        // ball_pos_y - 1 to make sure the first row is not part of the field
        if mutex_guard.speed.dy < 0.0 && ball_pos_y - 1 <= 0 + fabsf(floorf(mutex_guard.speed.dy)) as u8 {
            mutex_guard.speed.dy = -mutex_guard.speed.dy;
        }
        //////////////////////////////////////////////////////
        //                  Collision bot wall              //
        //////////////////////////////////////////////////////
        else if mutex_guard.speed.dy > 0.0 && ball_pos_y + floorf(mutex_guard.speed.dy) as u8 >= BUFFER_HEIGHT as u8 - 1 {
            mutex_guard.speed.dy = -mutex_guard.speed.dy;
            if mutex_guard.position.y as u8 >= BUFFER_HEIGHT as u8 - floorf(mutex_guard.speed.dy) as u8 {
                mutex_guard.position.y = mutex_guard.last_pos.y
            }
        }
        
    }
}


use super::GameConfig;

pub fn render_menu_text() {
    let halfway_point = BUFFER_WIDTH / 2;
    let mut writer = WRITER.lock();
    writer.write_string_at_pos(4, halfway_point - 2, "Pong!");
    writer.write_string_at_pos(6, halfway_point - 25, "Player1");
    writer.write_string_at_pos(7, halfway_point - 25, "move        : W S");
    writer.write_string_at_pos(8, halfway_point - 25, "change color: A D");

    writer.write_string_at_pos(10, halfway_point -25, "Player 2");
    writer.write_string_at_pos(11, halfway_point -25, "move        : Arrow Up   Arrow Down");
    writer.write_string_at_pos(12, halfway_point -25, "change color: Arrow Left Arrow Right");
    writer.write_string_at_pos(14, halfway_point - 13, "Press ESC to return to Menu");
    writer.write_string_at_pos(15, halfway_point - 15, "Press SPACEBAR to pause/unpause");
    writer.write_string_at_pos(17, halfway_point - 11, "Press SPACEBAR to start");
}

pub fn clear_menu_text() {
    let mut writer = WRITER.lock();
    let start_column = 0;
    let end_column = BUFFER_WIDTH -5; 
    (0..18).for_each(|row| {
        (start_column..end_column).for_each(|col| {
            writer.write_byte_at_pos(row, col, b' ');
        })
    })
}

pub fn render_pause_text() {
    WRITER.lock().write_string_at_pos(0, 0, "PAUSED");
}

pub fn clear_pause_text() {
    WRITER.lock().write_string_at_pos(0, 0, "      ");
}

pub fn reset_ball(ball: &mut PongBall) {
    let old_pos = ball.position;
    ball.position = crate::pongball::BallPosition::default();
    ball.last_pos = old_pos;
    ball.speed = crate::pongball::Speed::default();
    WRITER
        .lock()
        .write_byte_at_pos(floorf(old_pos.y) as usize, floorf(old_pos.x) as usize, b' ');
}

pub fn render_score(score: (u32, u32)) {
    let middle = BUFFER_WIDTH / 2;
    let (mut score_p1, mut score_p2) = score;

    // log10(2**32) + 1
    let max_digits_u32 = 10;

    let digits_in_p1 = libm::floorf(libm::log10f(score_p1 as f32)) as u32 + 1;

    let digits_in_p2 = libm::floorf(libm::log10f(score_p2 as f32)) as u32 + 1;

    {
        /*(0..digits_in_p1).map(|i| {
            score_p1 /= 10;
            (score_p1 % 10) as u8
        }).rev().for_each(|i| {
            crate::print!("{}", i);
        });*/
        let mut mutex_guard = WRITER.lock();

        (0..max_digits_u32).for_each(|i| {
            mutex_guard.write_byte_at_pos(0, middle - i as usize + 1, b' ');
        });

        (0..digits_in_p1).map(|i| {
            let s = (score_p1 % 10) as u8;
            score_p1 /= 10;
            (i, s)
        }).for_each(|(i,  s)| {
            mutex_guard.write_byte_at_pos(0, middle -i as usize - 1, 0x30 + s);
        });



        (0..max_digits_u32).for_each(|i| {
            mutex_guard.write_byte_at_pos(0, middle + i as usize + 1, b' ');
        });
    
        (0..digits_in_p2).map(|i| {
            let s = (score_p2 % 10) as u8;
            score_p2 /= 10;
            (i, s)
        }).rev().for_each(|(i, s)| {
            mutex_guard.write_byte_at_pos(0, middle + i as usize + 1, 0x30 + s)
        });

        /*(0..digits_in_p1).for_each(|i| {
            mutex_guard.write_byte_at_pos(0, middle - i as usize - 1, 0x30 + (score_p1 % 10) as u8);
            score_p1 /= 10;
        });

        (0..digits_in_p2).for_each(|i| {
            mutex_guard.write_byte_at_pos(0, middle + i as usize + 1, 0x30 + (score_p2 % 10) as u8);
            score_p2 /= 10;
        });*/

        mutex_guard.write_byte_at_pos(0, middle, b':');
    }
}

fn render_player(pos: &RwLock<Position>, color_code: &ColorCode) {
    let height = crate::vga_buffer::BUFFER_HEIGHT;
    let mut mutex_guard = WRITER.lock();
    let coords = pos.read();
    (0..height).for_each(|row| {
        mutex_guard.write_byte_at_pos(row as usize, coords.x as usize, b' ');
    });

    (coords.y - 2..coords.y + 3).for_each(|row| {
        mutex_guard.write_screen_char_at_pos(row as usize, coords.x as usize, ScreenChar { ascii_character: 0xfe, color_code: *color_code});
    })
}

fn render_ball(ball: &PongBall) {
    // let x = BUFFER_WIDTH as f32 / 2.0;
    // let y = BUFFER_HEIGHT as f32 / 2.0;
    // // WRITER.lock().write_byte_at_pos(10, 10, 0xfe);
    // println!(
    //     "x: {}\ny: {}",
    //     floorf(ball.position.x) as usize,
    //     floorf(ball.position.y) as usize
    // );
    WRITER.lock().write_byte_at_pos(
        floorf(ball.last_pos.y) as usize,
        floorf(ball.last_pos.x) as usize,
        b' ',
    );

    WRITER.lock().write_byte_at_pos(
        floorf(ball.position.y) as usize,
        floorf(ball.position.x) as usize,
        0x0040,
    )
}

pub enum Player {
    Player1,
    Player2,
}

#[derive(Debug)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}
