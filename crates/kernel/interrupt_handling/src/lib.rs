#![no_std]
#![feature(abi_x86_interrupt)]

pub mod handler;

pub unsafe fn init() {
    handler::init();
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SyscallResult {
    pub first: usize,
    pub second: usize,
}

#[link(name = "interrupt_handler_asm", kind = "static")]
extern "sysv64" {
    fn asm_perform_system_call(
        system_call_number: usize,
        a: usize,
        b: usize,
        c: usize,
        d: usize,
        e: usize,
    ) -> SyscallResult;
}

pub fn perform_system_call(
    system_call_number: usize,
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
) -> SyscallResult {
    unsafe { asm_perform_system_call(system_call_number, a, b, c, d, e) }
}
