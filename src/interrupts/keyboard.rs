use core::sync::atomic::Ordering;

use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;
use crate::state::{StateLocation::{Menu, Running, Paused}, render_menu_text, clear_menu_text, render_pause_text, clear_pause_text};
use crate::pongbar::Key;
use crate::{STATE, TIMER, STATE_LOCATION};
use pc_keyboard::{KeyCode, KeyEvent, KeyState};
use x86_64::instructions::port::Port;

use super::{InterruptIndex, PICS};


lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore)
    );
}


pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    TIMER.number.fetch_add(3, Ordering::Acquire);
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        match key_event {
            //////////////////////////////////////////////////////
            //                  Player1                         //
            //////////////////////////////////////////////////////
            KeyEvent {
                code: KeyCode::W,
                state: KeyState::Down,
            } => {
                let mut btn = STATE.player1.button_pressed.write();
                *btn = Key::Up;
            }
            KeyEvent {
                code: KeyCode::W,
                state: KeyState::Up,
            } => {
                //let mut btn = STATE.player1.button_pressed.write();
            }

            KeyEvent {
                code: KeyCode::S,
                state: KeyState::Down,
            } => {
                let mut btn = STATE.player1.button_pressed.write();
                *btn = Key::Down;
            }
            KeyEvent {
                code: KeyCode::S,
                state: KeyState::Up,
            } => {
                //let mut btn = STATE.player1.button_pressed.write();
            },
            KeyEvent {
                code: KeyCode::A,
                state: KeyState::Down,
            } => {
                if let Menu = crate::STATE_LOCATION.read() {
                    // Decrement Color
                    use crate::state::Player::Player1;
                    let prev_color = STATE.config.player_color(Player1);
                    STATE.config.set_color(prev_color.previous_color(), Player1);
                }
                
            },
            KeyEvent {
                code: KeyCode::D,
                state: KeyState::Down,
            } => {
                if let Menu = crate::STATE_LOCATION.read() {
                    // Increment Color
                    use crate::state::Player::Player1;
                    let prev_color = STATE.config.player_color(Player1);
                    STATE.config.set_color(prev_color.next_color(), Player1);
                }
                
            }

            //////////////////////////////////////////////////////
            //                  Player2                         //
            //////////////////////////////////////////////////////
            KeyEvent {
                code: KeyCode::ArrowUp,
                state: KeyState::Down,
            } => {
                let mut btn = STATE.player2.button_pressed.write();
                *btn = Key::Up;
            }
            KeyEvent {
                code: KeyCode::ArrowUp,
                state: KeyState::Up,
            } => {
                //let mut btn = STATE.player2.button_pressed.write();
            }

            KeyEvent {
                code: KeyCode::ArrowDown,
                state: KeyState::Down,
            } => {
                let mut btn = STATE.player2.button_pressed.write();
                *btn = Key::Down;
            }
            KeyEvent {
                code: KeyCode::ArrowDown,
                state: KeyState::Up,
            } => {
                //let mut btn = STATE.player2.button_pressed.write();
            },
            
            KeyEvent {
                code: KeyCode::Spacebar,
                state: KeyState::Down,
            } => {
                match STATE_LOCATION.read() {
                    Menu => {
                        clear_menu_text();
                        STATE_LOCATION.set(Running);
                    },
                    Running => {
                        render_pause_text();
                        STATE_LOCATION.set(Paused);
                    },
                    Paused => {
                        clear_pause_text();
                        STATE_LOCATION.set(Running);
                    } 

                }
            },
            KeyEvent {
                code: KeyCode::Spacebar,
                state: KeyState::Up,
            } => {

            },
            KeyEvent {
                code: KeyCode::ArrowRight,
                state: KeyState::Down,
            } => {
                if let Menu = crate::STATE_LOCATION.read() {
                    use crate::state::Player::Player2;
                    let prev_color = STATE.config.player_color(Player2);
                    STATE.config.set_color(prev_color.previous_color(), Player2);
                }
                
            },
            KeyEvent {
                code: KeyCode::ArrowLeft,
                state: KeyState::Down,
            } => {
                if let Menu = crate::STATE_LOCATION.read() {
                    use crate::state::Player::Player2;
                    let pre_color = STATE.config.player_color(Player2);
                    STATE.config.set_color(pre_color.next_color(), Player2);
                }
                
            },
            KeyEvent {
                code: KeyCode::Escape,
                state: KeyState::Down,
            } => {
                
                let current_location = crate::STATE_LOCATION.read();
                if current_location == Running || current_location == Paused {
                    crate::STATE.reset();
                }
                render_menu_text();

                crate::STATE_LOCATION.set(Menu);
            }



            _ => {
                // if let Some(key) = keyboard.process_keyevent(key_event) {
                // match key {
                //     DecodedKey::Unicode(character) => println!("{}", character),
                //     DecodedKey::RawKey(key) => println!("{:?}", key),
                // }
                // }
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}