use kernel_spin::KernelMutex;
use log::*;
use pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: KernelMutex<ChainedPics> =
    KernelMutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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
        pic.initialize();
        pic.set_mask(0, !0x03);
        pic.set_mask(1, 0xff);
        trace!("pic initialized");
    });
}
