#![no_std]

use core::slice::from_raw_parts_mut;
use log::*;
use page_management::physical::{
    map::PhysicalMemoryMap, page_usage::PageUsageRawType,
};
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

pub struct InitializedKernelArguments {
    pub st: SystemTable<Runtime>,
}

impl KernelArguments {
    #[inline(never)]
    pub fn init(self) -> InitializedKernelArguments {
        unsafe {
            page_management::page_table::initialize_identity_base(
                self.identity_base,
            );
        }

        // TODO self is pretty hacky. The arguments should probably not contain any pointers, but physical addresses
        unsafe {
            let (buffer, base) = self.physical_memory_map.release();
            let pointer = buffer.as_mut_ptr();

            let address: VirtAddr = VirtAddr::from_ptr(pointer)
                + self.identity_base.start_address().as_u64();
            let new_pointer = address.as_mut_ptr::<PageUsageRawType>();

            let new_buffer = from_raw_parts_mut(new_pointer, buffer.len());

            let physical_memory_map =
                PhysicalMemoryMap::from_raw_parts(new_buffer, base);

            physical_memory_map.register_global();
        }

        unsafe {
            // Adjust the per core local storage

            let address =
                VirtAddr::from_ptr(cpu_local_storage::read_raw::<()>());
            cpu_local_storage::init_raw(
                (address + self.identity_base.start_address().as_u64())
                    .as_mut_ptr::<()>(),
            );
        }

        serial_io::logger::init();
        log::set_max_level(LevelFilter::Info);

        info!("KernelArguments initialized");

        InitializedKernelArguments { st: self.st }
    }
}
