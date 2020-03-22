#![no_std]

use core::slice::from_raw_parts_mut;
use page_usage::{PageUsageRawType, PhysicalMemoryMap};
use uefi::table::{Runtime, SystemTable};
use x86_64::{structures::paging::Page, VirtAddr};

#[repr(C)]
pub struct KernelArguments {
    pub st: SystemTable<Runtime>,

    pub physical_memory_map: PhysicalMemoryMap<'static>,
    pub identity_base: Page,
}

impl KernelArguments {
    pub fn init(self) -> Self {
        unsafe {
            page_table::initialize(self.identity_base);
        };

        // TODO this is pretty hacky. The arguments should probably not contain any pointers, but physical addresses
        let physical_memory_map = unsafe {
            let (buffer, base) = self.physical_memory_map.release();
            let pointer = buffer.as_mut_ptr();

            let address: VirtAddr = VirtAddr::from_ptr(pointer)
                + self.identity_base.start_address().as_u64();
            let new_pointer = address.as_mut_ptr::<PageUsageRawType>();

            let new_buffer = from_raw_parts_mut(new_pointer, buffer.len());

            PhysicalMemoryMap::from_raw_parts(new_buffer, base)
        };

        KernelArguments {
            physical_memory_map,
            ..self
        }
    }
}

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}
