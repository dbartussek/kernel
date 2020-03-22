#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

pub mod alloc_utils;
pub mod load_elf;
pub mod memory_map;
pub mod read_kernel;

use crate::{
    load_elf::load_elf, memory_map::exit_boot_services,
    read_kernel::read_kernel,
};
use kernel_core::KernelArguments;
use log::*;
use page_table::KernelPageTable;
use page_usage::PageUsage;
use uefi::prelude::*;
use x86_64::{
    structures::paging::{Mapper, Page, PageTableFlags},
    VirtAddr,
};

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    let mut physical_memory_map = memory_map::create_physical_memory_map(&st);

    let identity_base =
        Page::from_start_address(VirtAddr::new(0xffff800000000000)).unwrap();

    let (_loaded_kernel, kernel_entry) =
        load_elf(&kernel, st.boot_services(), identity_base);

    info!("Exiting boot services");

    let st = exit_boot_services(image, st, &mut physical_memory_map);

    let mut page_table = unsafe {
        KernelPageTable::initialize_and_create(
            Page::from_start_address(VirtAddr::new(0)).unwrap(),
            &mut physical_memory_map,
            Page::from_start_address(VirtAddr::new(0)).unwrap(),
        )
    };

    if identity_base.start_address().as_u64() != 0 {
        page_table.get_manager().map_range(
            physical_memory_map.physical_range(),
            identity_base,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            false,
            &mut physical_memory_map
                .frame_allocator(PageUsage::PageTable { reference_count: 0 }),
        );
    }

    unsafe {
        page_table.activate();
    };

    assert_eq!(Some(0), unsafe {
        page_table
            .get_page_table_mut()
            .translate_page(identity_base)
            .ok()
            .map(|frame| frame.start_address().as_u64())
    });

    kernel_entry(KernelArguments {
        st,
        physical_memory_map,
        identity_base,
    });

    panic!("Kernel returned");
}
