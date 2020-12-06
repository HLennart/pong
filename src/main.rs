#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}


mod state;
mod interrupts;
mod pongball;
mod pongbar;
mod ui;
mod vga_buffer;
mod state_location;

use ui::Timer;
use state::{GameState, GlobalStateLocation, render_score};


// Globals
lazy_static! {
    pub static ref STATE_LOCATION: GlobalStateLocation = GlobalStateLocation::default();
}

lazy_static! {
    pub static ref STATE: GameState = {
        x86_64::instructions::hlt();
        GameState::new()
    };
}

lazy_static! {
    pub static ref TIMER: Timer = Timer::new();
}

use core::{fmt::Write, panic::PanicInfo};
use lazy_static::lazy_static;





#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    {
        let score = STATE.score.read();
        render_score(*score);
    }


    hlt_loop();
}

// Looping through the [hlt instruction](https://en.wikipedia.org/wiki/HLT_(x86_instruction))
// uses a lot less resouces than looping around a no-op (e.g. `()`)
fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    write!(vga_buffer::WRITER.lock(), "panic: {}", info).unwrap_or(());
    hlt_loop();
}

pub fn init() {
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
