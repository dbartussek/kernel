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

impl RegisterContext {
    pub fn system_call_number(&self) -> usize {
        self.rdi
    }

    pub fn system_call_arg_a(&self) -> usize {
        self.rsi
    }
    pub fn system_call_arg_b(&self) -> usize {
        self.rdx
    }
    pub fn system_call_arg_c(&self) -> usize {
        self.rcx
    }
    pub fn system_call_arg_d(&self) -> usize {
        self.r8
    }
    pub fn system_call_arg_e(&self) -> usize {
        self.r9
    }

    pub fn return_value_a(&mut self, value: usize) -> &mut Self {
        self.rax = value;
        self
    }
    pub fn return_value_b(&mut self, value: usize) -> &mut Self {
        self.rdx = value;
        self
    }
}

#[link(name = "interrupt_handler_asm", kind = "static")]
extern "x86-interrupt" {
    pub fn asm_breakpoint_handler(stack_frame: &mut InterruptStackFrame);
    pub fn asm_int_syscall_handler(stack_frame: &mut InterruptStackFrame);
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub unsafe fn init() {
    IDT.breakpoint.set_handler_fn(core::mem::transmute(
        asm_breakpoint_handler as *mut (),
    ));

    IDT.double_fault.set_handler_fn(double_fault_handler);

    IDT[0x80].set_handler_fn(core::mem::transmute(
        asm_int_syscall_handler as *mut (),
    ));

    IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(
    frame: &mut InterruptStackFrame,
    _code: u64,
) -> ! {
    panic!("Double fault\n{:#?}", frame)
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

#[no_mangle]
pub extern "sysv64" fn rust_int_syscall_handler(
    stack_frame: &mut InterruptStackFrame,
    register_context: &mut RegisterContext,
) {
    trace!("Syscall:\n{:#?}\n{:#X?}", stack_frame, register_context);

    register_context.return_value_a(0x42);
    register_context.return_value_b(0x21);
}
