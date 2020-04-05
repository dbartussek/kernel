use local_apic::Registers;
use log::*;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn spurious_interrupt_handler(
    _frame: &mut InterruptStackFrame,
) {
}

pub extern "x86-interrupt" fn apic_timer_handler(
    _frame: &mut InterruptStackFrame,
) {
    unsafe {
        Registers::global().end_of_interrupt();
    }
}
