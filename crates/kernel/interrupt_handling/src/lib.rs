#![no_std]
#![feature(abi_x86_interrupt)]

use log::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[repr(C)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RegisterContext {
    pub rax: usize,
    pub rbx: usize,
    pub rcx: usize,
    pub rdx: usize,

    pub rbp: usize,

    pub rsi: usize,
    pub rdi: usize,

    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
}

#[link(name = "interrupt_handler_asm", kind = "static")]
extern "x86-interrupt" {
    pub fn asm_breakpoint_handler(stack_frame: &mut InterruptStackFrame);
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub unsafe fn init() {
    IDT.breakpoint.set_handler_fn(core::mem::transmute(
        asm_breakpoint_handler as *mut (),
    ));
    IDT.load();
}

#[no_mangle]
pub extern "sysv64" fn rust_breakpoint_handler(
    stack_frame: &mut InterruptStackFrame,
    register_context: &mut RegisterContext,
) {
    trace!(
        "Interrupt BREAKPOINT\n{:#?}\n{:#X?}",
        stack_frame,
        register_context
    );
}
