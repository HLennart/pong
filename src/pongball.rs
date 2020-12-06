use core::sync::atomic::Ordering;

use x86_64::instructions::interrupts;

use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH};


pub static LOWEST_SPEED: f32 = 0.5;
pub static HIGHEST_SPEED: f32 = 10.0;

pub struct PongBall {
    pub last_pos: BallPosition,
    pub position: BallPosition,
    pub speed: Speed,
}

impl PongBall {
    pub fn new() -> Self {
        Self {
            last_pos: BallPosition {
                x: (BUFFER_WIDTH as f32 / 2.0) - 1.0,
                y: (BUFFER_HEIGHT as f32 / 2.0) - 1.0,
            },
            position: BallPosition {
                x: BUFFER_WIDTH as f32 / 2.0,
                y: BUFFER_HEIGHT as f32 / 2.0,
            },
            speed: Speed::default(),
        }
    }

    pub fn move_ball(&mut self) {
        self.last_pos = self.position;

        self.position.x += self.speed.dx;
        self.position.y += self.speed.dy;
        //crate::println!("pos: {:?}\nspeed: {:?}", self.position, self.speed);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BallPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for BallPosition {
    fn default() -> Self {
        Self {
            x: BUFFER_WIDTH as f32 / 2.0,
            y: BUFFER_HEIGHT as f32 / 2.0,
        }
    }
}

#[derive(Debug)]
pub struct Speed {
    pub dx: f32,
    pub dy: f32,
}

impl Default for Speed {
    fn default() -> Self {
        
        get_random_start_speeds()
    }
}

fn get_random_start_speeds() -> Speed {
    use mish::prelude::f32::{acos, cos};
    
    interrupts::without_interrupts(|| {

        let random_u32: u8 = crate::TIMER.number.load(Ordering::Relaxed) as u8;
        // "Random" numbers between 0 and 2
        let mut rng = random_u32 as f32 / (u8::MAX as f32 / 2.0);
        // "Random numbers between -1 and 1"
        rng -= 1.0;

        let mut x: f32 = cos((rng) * core::f32::consts::PI);
        let y: f32 = acos(x);
        if x >= 0.0 && x < 0.3 {
            x += 0.4;
        }
        else
        if x <= 0.0 && x > -0.3 {
            x -= 0.4;
        }
            

        // Speed parameters
        // Might make these accessible in the menu later on
        // TODO: ^
        Speed {
            dx: 3.0 * x ,
            dy: 0.8 * y ,
        }
    })
}
    