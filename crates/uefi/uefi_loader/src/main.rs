#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]

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
use core::mem::{ManuallyDrop, MaybeUninit};
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

// These are duplicated from the ffi/stack_switch crate.
// For some reason, the "intel" modifier is ignored when the function is not in the *exact* same file
// it is used in. I am investigating this, but for now, this works.

pub unsafe fn call_with_stack<T>(
    arg: &mut T,
    function: extern "sysv64" fn(&mut T) -> (),
    stack: *mut u8,
) {
    asm!(r#"
    mov rbp, rsp
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

/// Calls a closure and returns the result
///
/// This function is unsafe because it changes the stack pointer to stack.
/// stack must be suitable to be used as a stack pointer on the target system.
pub unsafe fn call_closure_with_stack<F, R>(closure: F, stack: *mut u8) -> R
where
    F: FnOnce() -> R,
{
    extern "sysv64" fn inner<F, R>(data: &mut (ManuallyDrop<F>, MaybeUninit<R>))
    where
        F: FnOnce() -> R,
    {
        let result = {
            // Read the closure from context, taking ownership of it
            let function = unsafe { ManuallyDrop::take(&mut data.0) };

            // Call the closure.
            // This consumes it and returns the result
            function()
        };

        // Write the result into the context
        data.1 = MaybeUninit::new(result);
    }

    // The context contains the closure and uninitialized memory for the return value
    let mut context = (ManuallyDrop::new(closure), MaybeUninit::uninit());

    call_with_stack(
        &mut context,
        // We create a new, internal function that does not close over anything
        // and takes a context reference as its argument
        inner,
        stack,
    );

    // Read the result from the context
    // No values are in the context anymore afterwards
    context.1.assume_init()
}


#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

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

    let stack_top: VirtAddr = {
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

        stack_top.start_address()
    };

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

    unsafe {
        call_closure_with_stack(
            || {
                // Call into kernel
                kernel_entry(KernelArguments {
                    st,
                    physical_memory_map,
                    identity_base,
                });

                exit(-2)
            },
            stack_top.as_mut_ptr(),
        )
    };

    exit(-2)
}
