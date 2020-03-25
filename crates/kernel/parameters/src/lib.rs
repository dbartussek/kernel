#![no_std]

use core::slice::from_raw_parts_mut;
use log::*;
use page_usage::{PageUsageRawType, PhysicalMemoryMap};
use uefi::table::{Runtime, SystemTable};
use x86_64::{structures::paging::Page, VirtAddr};

pub type KernelEntrySignature =
    unsafe extern "sysv64" fn(*mut KernelArguments) -> ();

#[repr(C)]
pub struct KernelArguments {
    pub st: SystemTable<Runtime>,

    pub physical_memory_map: PhysicalMemoryMap<'static>,
    pub identity_base: Page,
}

impl KernelArguments {
    #[inline(never)]
    pub fn init(self) -> Self {
        #[inline(never)]
        fn force_move_value<T>(value: T) -> T {
            value
        }

        let this = force_move_value(self);

        unsafe {
            page_table::initialize(this.identity_base);
        };

        // TODO this is pretty hacky. The arguments should probably not contain any pointers, but physical addresses
        let physical_memory_map = unsafe {
            let (buffer, base) = this.physical_memory_map.release();
            let pointer = buffer.as_mut_ptr();

            let address: VirtAddr = VirtAddr::from_ptr(pointer)
                + this.identity_base.start_address().as_u64();
            let new_pointer = address.as_mut_ptr::<PageUsageRawType>();

            let new_buffer = from_raw_parts_mut(new_pointer, buffer.len());

            PhysicalMemoryMap::from_raw_parts(new_buffer, base)
        };

        serial_io::logger::init();
        log::set_max_level(LevelFilter::Info);

        info!("KernelArguments initialized");

        KernelArguments {
            physical_memory_map,
            ..this
        }
    }
}
