#![no_std]

use page_usage::PhysicalMemoryMap;
use uefi::table::{Runtime, SystemTable};
use x86_64::structures::paging::Page;

#[repr(C)]
pub struct KernelArguments {
    pub st: SystemTable<Runtime>,

    pub physical_memory_map: PhysicalMemoryMap<'static>,
    pub identity_base: Page,
}

impl KernelArguments {
    pub fn init(&mut self) {
        unsafe {
            page_table::initialize(self.identity_base);
        };
    }
}

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}
