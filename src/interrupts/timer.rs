use core::sync::atomic::Ordering;

use x86_64::structures::idt::InterruptStackFrame;

use super::InterruptIndex;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    {
        use crate::{STATE, TIMER, STATE_LOCATION};
        use crate::state::StateLocation::{Menu, Running, Paused};
        let mut number = TIMER.number.fetch_add(1, Ordering::Acquire);
        if number == 0 {
            number = 1;
        }

        match STATE_LOCATION.read() {
            Menu => {
                if number % 3 == 0 {
                    STATE.show_menu()
                }
             } ,
            Running => {
                if number % 3 == 0 {
                    STATE.step();
                }
            },
            Paused => ()
        }

        
    }

    unsafe {
        super::PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}