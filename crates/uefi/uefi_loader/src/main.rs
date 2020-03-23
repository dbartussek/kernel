#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(maybe_uninit_extra)]

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
use alloc::boxed::Box;
use core::mem::MaybeUninit;
use kernel_core::{exit, KernelArguments};
use log::*;
use page_table::KernelPageTable;
use page_usage::PageUsage;
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};
use x86_64::{
    registers::{control::EferFlags, model_specific::Efer},
    structures::paging::{
        frame::PhysFrameRange, Mapper, Page, PageTableFlags, PhysFrame,
        Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

const KERNEL_ADDRESS_SPACE_BASE: u64 = 0xffff800000000000;
const KERNEL_REGION_SIZE: u64 = 0x100000000000;

const IDENTITY_BASE: u64 = KERNEL_ADDRESS_SPACE_BASE + KERNEL_REGION_SIZE * 0;
const STACK_BASE: u64 = KERNEL_ADDRESS_SPACE_BASE + KERNEL_REGION_SIZE * 1;

const STACK_SIZE_PAGES: u64 = 256;

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    unsafe {
        Efer::write(Efer::read() | EferFlags::NO_EXECUTE_ENABLE);
    };

    let kernel = read_kernel(&st);
    info!("Kernel loaded: {} bytes", kernel.len());

    let mut physical_memory_map = memory_map::create_physical_memory_map(&st);

    let identity_base =
        Page::from_start_address(VirtAddr::new(IDENTITY_BASE)).unwrap();

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

    let stack_top: Page<Size4KiB> = {
        let stack_base_frame =
            PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(
                st.boot_services()
                    .allocate_pages(
                        AllocateType::AnyPages,
                        MemoryType::LOADER_DATA,
                        STACK_SIZE_PAGES as usize,
                    )
                    .unwrap()
                    .log(),
            ))
            .unwrap();
        let stack_top_frame = stack_base_frame + STACK_SIZE_PAGES;

        let stack_base: Page<Size4KiB> =
            Page::from_start_address(VirtAddr::new(STACK_BASE)).unwrap();
        let stack_top = stack_base + STACK_SIZE_PAGES;

        info!(
            "Allocated {} pages for stack, stack_top: 0x{:X}",
            STACK_SIZE_PAGES,
            stack_top_frame.start_address().as_u64()
        );

        page_table.get_manager().map_range(
            PhysFrameRange {
                start: stack_base_frame,
                end: stack_top_frame,
            },
            Page::from_start_address(VirtAddr::new(STACK_BASE)).unwrap(),
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE,
            false,
            unsafe {
                &mut physical_memory_map.external_frame_allocator(
                    PageUsage::KernelStack { thread: 0 },
                    |_| uefi_frame_allocator(st.boot_services())(),
                )
            },
        );

        info!(
            "Mapped stack to 0x{:X}",
            stack_base.start_address().as_u64()
        );

        stack_top
    };

    let kernel_arguments_box: &'static mut MaybeUninit<KernelArguments> =
        Box::leak(Box::new(MaybeUninit::zeroed()));

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

    assert!(unsafe {
        page_table
            .get_page_table_mut()
            .translate_page(stack_top - 1u64)
            .is_ok()
    });
    assert!(unsafe {
        page_table
            .get_page_table_mut()
            .translate_page(Page::<Size4KiB>::containing_address(
                VirtAddr::from_ptr(kernel_entry as *const ()),
            ))
            .is_ok()
    });

    unsafe {
        pub unsafe fn call_with_stack<T>(
            arg: *mut T,
            function: extern "sysv64" fn(*mut T) -> (),
            stack: *mut u8,
        ) {
            asm!(r#"
                mov rbp, rsp

                and $2, -16
                mov rsp, $2

                call $1

                mov rsp, rbp
                "#
                : // Return values
                : "{rdi}"(arg), "r"(function), "r"(stack) // Arguments
                : "rbp", "cc", "memory" // Clobbers
                : "volatile", "intel" // Options
            );
        }

        let kernel_arguments = kernel_arguments_box.write(KernelArguments {
            st,
            physical_memory_map,
            identity_base,
        }) as *mut KernelArguments;
        let kernel_arguments = (VirtAddr::from_ptr(kernel_arguments)
            + identity_base.start_address().as_u64())
        .as_mut_ptr::<KernelArguments>();

        call_with_stack(
            kernel_arguments,
            kernel_entry,
            stack_top.start_address().as_mut_ptr(),
        );
    };

    exit(-2)
}
