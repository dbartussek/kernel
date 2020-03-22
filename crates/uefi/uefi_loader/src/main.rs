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
use uefi::prelude::*;
use x86_64::{structures::paging::Page, VirtAddr};

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    let (_loaded_kernel, kernel_entry) = load_elf(&kernel, st.boot_services());

    let mut physical_memory_map = memory_map::create_physical_memory_map(&st);

    let st = exit_boot_services(image, st, &mut physical_memory_map);

    let identity_base = Page::from_start_address(VirtAddr::new(0)).unwrap();

    let page_table = unsafe {
        KernelPageTable::initialize_and_create(
            identity_base,
            &mut physical_memory_map,
            Page::from_start_address(VirtAddr::new(0)).unwrap(),
        )
    };

    unsafe {
        page_table.activate();
    };

    kernel_entry(KernelArguments {
        st,
        physical_memory_map,
        identity_base,
        page_table,
    });

    panic!("Kernel returned");
}
