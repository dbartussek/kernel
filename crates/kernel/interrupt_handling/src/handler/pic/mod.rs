use crate::handler::pic::schedule::pump_tasks;
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use kernel_spin::KernelMutex;
use log::*;
use pic8259::ChainedPics;
use pit::Pit;
use x86_64::structures::idt::InterruptStackFrame;

pub mod schedule;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: KernelMutex<ChainedPics> =
    KernelMutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub static PIT: KernelMutex<Pit> = KernelMutex::new(Pit::new());

static PIT_TIME_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub unsafe fn init() {
    PICS.lock(|pic| {
        // Disable all interrupts
        pic.initialize();
        pic.set_mask(0, 0xff);
        pic.set_mask(1, 0xff);
        trace!("pic initialized");
    });

    PIT.lock(|pit| pit.init());
}

pub fn get_duration() -> Duration {
    Duration::from_nanos(PIT_TIME_COUNTER.load(Ordering::Acquire))
}

pub extern "x86-interrupt" fn pic_timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    PIT_TIME_COUNTER.fetch_add(
        PIT.lock(|pit| pit.duration()).as_nanos() as u64,
        Ordering::SeqCst,
    );

    pump_tasks();

    unsafe {
        PICS.lock(|pics| {
            pics.notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
        });
    }
}

pub extern "x86-interrupt" fn apic_timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    PIT_TIME_COUNTER.fetch_add(
        PIT.lock(|pit| pit.duration()).as_nanos() as u64,
        Ordering::SeqCst,
    );

    pump_tasks();

    unsafe {
        PICS.lock(|pics| {
            pics.notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
        });
    }
}
