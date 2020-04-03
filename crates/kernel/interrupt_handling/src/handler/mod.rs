use crate::handler::pic::{timer_interrupt_handler, InterruptIndex};
use log::*;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

pub(crate) mod gdt;
pub(crate) mod pic;

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
    gdt::init();

    IDT.breakpoint.set_handler_fn(core::mem::transmute(
        asm_breakpoint_handler as *mut (),
    ));

    IDT.double_fault.set_handler_fn(double_fault_handler);

    IDT.page_fault.set_handler_fn(page_fault_handler);
    IDT.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);

    {
        // pic8259 interrupts
        macro_rules! add_unknown_handler {
            ($name: ident, $id: expr) => {
                extern "x86-interrupt" fn $name(
                    _stack_frame: &mut InterruptStackFrame,
                ) {
                    panic!(concat!(
                        "Unhandled pic interrupt ",
                        stringify!($id)
                    ));
                }

                IDT[InterruptIndex::Timer.as_usize() + $id]
                    .set_handler_fn($name);
            };
        };

        add_unknown_handler!(pic1, 1);
        add_unknown_handler!(pic2, 2);
        add_unknown_handler!(pic3, 3);
        add_unknown_handler!(pic4, 4);
        add_unknown_handler!(pic5, 5);
        add_unknown_handler!(pic6, 6);
        add_unknown_handler!(pic7, 7);
        add_unknown_handler!(pic8, 8);
        add_unknown_handler!(pic9, 9);
        add_unknown_handler!(pic10, 10);
        add_unknown_handler!(pic11, 11);
        add_unknown_handler!(pic12, 12);
        add_unknown_handler!(pic13, 13);
        add_unknown_handler!(pic14, 14);
        add_unknown_handler!(pic15, 15);

        IDT[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
    }

    IDT[0x80].set_handler_fn(core::mem::transmute(
        asm_int_syscall_handler as *mut (),
    ));

    IDT.load();

    pic::init();
}

extern "x86-interrupt" fn double_fault_handler(
    frame: &mut InterruptStackFrame,
    _code: u64,
) -> ! {
    panic!("Double fault\n{:#?}", frame)
}

extern "x86-interrupt" fn page_fault_handler(
    frame: &mut InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    panic!("Page fault: {:?}\n{:#?}", code, frame)
}
extern "x86-interrupt" fn general_protection_fault_handler(
    frame: &mut InterruptStackFrame,
    code: u64,
) {
    panic!("General protection fault: 0x{:X}\n{:#?}", code, frame)
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
