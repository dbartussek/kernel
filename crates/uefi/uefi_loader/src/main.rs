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
use kernel_core::{exit, KernelArguments};
use log::*;
use page_table::KernelPageTable;
use page_usage::PageUsage;
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};
use x86_64::{
    structures::paging::{
        Mapper, Page, PageTableFlags, PhysFrame, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
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

    fn uefi_frame_allocator<'lt>(
        bt: &'lt BootServices,
    ) -> impl 'lt + Fn() -> Option<UnusedPhysFrame> {
        move || {
            bt.allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                1,
            )
            .ok()
            .map(|address| unsafe {
                UnusedPhysFrame::new(PhysFrame::containing_address(
                    PhysAddr::new(address.log()),
                ))
            })
        }
    }

    // Create page table
    let mut page_table = unsafe {
        KernelPageTable::initialize_and_create(
            &mut physical_memory_map,
            Page::from_start_address(VirtAddr::new(0)).unwrap(),
            |_| uefi_frame_allocator(st.boot_services())(),
        )
    };

    // Add high half mapping
    if identity_base.start_address().as_u64() != 0 {
        page_table.get_manager().map_range(
            physical_memory_map.physical_range(),
            identity_base,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            false,
            unsafe {
                &mut physical_memory_map.external_frame_allocator(
                    PageUsage::PageTable { reference_count: 0 },
                    |_| uefi_frame_allocator(st.boot_services())(),
                )
            },
        );
    }

    info!(
        "Pages used for page tables: 0x{:X}",
        physical_memory_map
            .iter()
            .filter(|entry| match entry {
                PageUsage::PageTable { .. }
                | PageUsage::PageTableRoot { .. } => true,
                _ => false,
            })
            .count()
    );
    info!(
        "Available pages: 0x{:X}",
        physical_memory_map
            .iter()
            .filter(|entry| *entry == PageUsage::Empty)
            .count()
    );

    info!("Exiting boot services");

    let st = exit_boot_services(image, st, &mut physical_memory_map);

    // Activate the new page table
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

    // Call into kernel
    kernel_entry(KernelArguments {
        st,
        physical_memory_map,
        identity_base,
    });

    exit(-2);
}
