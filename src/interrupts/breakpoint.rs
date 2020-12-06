use x86_64::structures::idt::InterruptStackFrame;
use crate::print;

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    print!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}