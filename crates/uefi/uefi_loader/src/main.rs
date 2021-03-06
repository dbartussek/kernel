#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(maybe_uninit_extra)]

#[macro_use]
extern crate alloc;

pub mod alloc_utils;
pub mod memory_map;
pub mod read_kernel;

use crate::{memory_map::exit_boot_services, read_kernel::read_kernel};
use acpi::{RootSystemDescriptionPointer2, RSDP2_GUID};
use alloc::boxed::Box;
use call_with_stack::call_with_stack;
use core::mem::MaybeUninit;
use cpu_local_storage::data::{CoreId, CpuLocalData};
use elf_loader::parameters::AdHocLoadParameters;
use log::*;
use page_management::{
    page_table::{
        identity_page,
        managed_page_table::{
            ManagedPageTable, ModificationFlags, IDENTITY_BASE,
            KERNEL_STACK_BASE,
        },
    },
    physical::{map::PhysicalMemoryMap, page_usage::PageUsage},
};
use parameters::{KernelArguments, KernelEntrySignature};
use uefi::{
    prelude::*,
    table::boot::{AllocateType, MemoryType},
};
use x86_64::{
    registers::{control::EferFlags, model_specific::Efer},
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, Page, PageTable,
        PageTableFlags, PhysFrame, Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

const STACK_SIZE_PAGES: usize = 256;

fn uefi_frame_allocator<'lt>(
    bt: &'lt BootServices,
) -> impl 'lt + Fn() -> Option<UnusedPhysFrame> {
    move || {
        bt.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .ok()
            .map(|address| unsafe {
                UnusedPhysFrame::new(PhysFrame::containing_address(
                    PhysAddr::new(address.log()),
                ))
            })
    }
}

unsafe fn setup_page_table<A>(
    desired_identity_base: Page<Size4KiB>,
    mut allocate: A,
) -> ManagedPageTable
where
    A: FnMut(&PhysicalMemoryMap) -> Option<UnusedPhysFrame>,
{
    let (physical_base, physical_range) =
        PhysicalMemoryMap::global(|physical_memory_map| {
            let physical_base = physical_memory_map.base();
            assert_eq!(physical_base.start_address().as_u64(), 0);

            let physical_range = physical_memory_map.physical_range();

            (physical_base, physical_range)
        });

    let physical_range = (0usize
        ..((physical_range.end - physical_range.start) as usize))
        .map(move |index| physical_base + (index as u64));

    fn create_page_table<A>(allocator: &mut A) -> Option<PhysFrame>
    where
        A: FrameAllocator<Size4KiB>,
    {
        allocator.allocate_frame().map(|frame| {
            let frame = frame.frame();
            {
                let table = identity_page(frame)
                    .start_address()
                    .as_mut_ptr::<PageTable>();
                unsafe { table.write(PageTable::new()) };
            }
            frame
        })
    }

    let root = PhysicalMemoryMap::global(|physical_memory_map| {
        // Allocate the root page
        let root = create_page_table(
            &mut physical_memory_map.external_frame_allocator(
                PageUsage::PageTableRoot,
                &mut allocate,
            ),
        )
        .unwrap();

        let level_4_table = &mut *identity_page(root)
            .start_address()
            .as_mut_ptr::<PageTable>();

        // Fill the kernel space top level entries
        for entry in level_4_table.iter_mut().skip(512 / 2) {
            let page = create_page_table(
                &mut physical_memory_map.external_frame_allocator(
                    PageUsage::PageTable,
                    &mut allocate,
                ),
            )
            .unwrap();
            entry.set_frame(
                page,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
        }

        root
    });

    let mut kernel_page_table = ManagedPageTable::from_raw_frame(root);
    kernel_page_table.modify(
        ModificationFlags {
            user_space: true,
            identity: true,
            kernel_stack: false,
            kernel_heap: false,
        },
        |manager| {
            // Map all physical pages to their identity position:
            // - In low addresses (for bootloader)
            manager
                .map_pages_external_frame_allocator(
                    Page::from_start_address(VirtAddr::new(0)).unwrap(),
                    physical_range.clone(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    false,
                    &mut allocate,
                )
                .unwrap();

            // - In high addresses (for kernel)
            manager
                .map_pages_external_frame_allocator(
                    desired_identity_base,
                    physical_range,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    false,
                    &mut allocate,
                )
                .unwrap();
        },
    );

    kernel_page_table
}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout().reset(false).unwrap().log();

    info!("Initialized");

    let rsdp = {
        let acpi_2_rsdp = st
            .config_table()
            .iter()
            .find(|table| table.guid == RSDP2_GUID)
            .expect("ACPI 2.0 RSDP is missing");

        unsafe {
            let ptr: *const RootSystemDescriptionPointer2 =
                core::mem::transmute(acpi_2_rsdp.address);
            let ptr = &*ptr;
            assert!(ptr.check_valid());
        }

        PhysAddr::new(acpi_2_rsdp.address as usize as u64)
    };

    {
        // Initialize the CpuLocalData.
        // This has to be done early, as there are assertions in locks that will access this data.

        let cpu_local_data = Box::new(CpuLocalData {
            core_id: CoreId::from_optional_full_id(1).unwrap(),
        });
        let cpu_local_data = Box::leak(cpu_local_data);
        unsafe { cpu_local_storage::init_raw(cpu_local_data as *mut _) };
    }

    unsafe {
        page_management::page_table::initialize_identity_base(
            Page::from_start_address(VirtAddr::new(0)).unwrap(),
        );
    }
    let desired_identity_base =
        Page::<Size4KiB>::from_start_address(VirtAddr::new(IDENTITY_BASE))
            .unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::NO_EXECUTE_ENABLE);
    };

    let physical_memory_map = memory_map::create_physical_memory_map(&st);
    unsafe {
        physical_memory_map.register_global();
    }

    let kernel = {
        let kernel_data = read_kernel(&st);
        info!("Kernel loaded: {} bytes", kernel_data.len());

        elf_loader::load(
            &kernel_data,
            AdHocLoadParameters {
                allocate: |size| {
                    let address = st
                        .boot_services()
                        .allocate_pages(
                            AllocateType::AnyPages,
                            MemoryType::LOADER_DATA,
                            size,
                        )
                        .ok()?
                        .log();

                    let memory = VirtAddr::new(address);
                    let virtual_pages = memory + IDENTITY_BASE;

                    info!("Kernel virtual base: {:?}", virtual_pages);

                    let memory =
                        Page::<Size4KiB>::from_start_address(memory).unwrap();
                    let virtual_pages =
                        Page::<Size4KiB>::from_start_address(virtual_pages)
                            .unwrap();

                    let size = size as u64;

                    Some((
                        PageRange {
                            start: memory,
                            end: memory + size,
                        },
                        PageRange {
                            start: virtual_pages,
                            end: virtual_pages + size,
                        },
                    ))
                },
                deallocate: |_pages| unimplemented!(),
                set_permissions: |_pages, _permissions| unimplemented!(),
            },
        )
    };

    info!("Kernel entry: {:x?}", kernel.entry.as_ptr::<()>());

    // Create page table
    let mut page_table = unsafe {
        setup_page_table(desired_identity_base, |_| {
            uefi_frame_allocator(st.boot_services())()
        })
    };

    info!("Set up new page table");

    info!(
        "Pages used for page tables: 0x{:X}",
        PhysicalMemoryMap::global(|map| map
            .iter()
            .filter(|entry| match entry {
                PageUsage::PageTable { .. }
                | PageUsage::PageTableRoot { .. } => true,
                _ => false,
            })
            .count())
    );
    info!(
        "Available pages: 0x{:X}",
        PhysicalMemoryMap::global(|map| map
            .iter()
            .filter(|entry| *entry == PageUsage::Empty)
            .count())
    );

    let stack_top: Page<Size4KiB> = {
        let stack_base: Page<Size4KiB> =
            Page::from_start_address(VirtAddr::new(KERNEL_STACK_BASE)).unwrap();
        let stack_top = stack_base + (STACK_SIZE_PAGES as u64);

        unsafe {
            page_table.modify(
                ModificationFlags {
                    kernel_stack: true,
                    ..Default::default()
                },
                |manager| {
                    manager
                        .map_pages_external_frame_allocator(
                            Page::from_start_address(VirtAddr::new(
                                KERNEL_STACK_BASE,
                            ))
                            .unwrap(),
                            (0..STACK_SIZE_PAGES).map(|_| {
                                PhysFrame::<Size4KiB>::from_start_address(
                                    PhysAddr::new(
                                        st.boot_services()
                                            .allocate_pages(
                                                AllocateType::AnyPages,
                                                MemoryType::LOADER_DATA,
                                                1,
                                            )
                                            .unwrap()
                                            .log(),
                                    ),
                                )
                                .unwrap()
                            }),
                            PageTableFlags::PRESENT
                                | PageTableFlags::WRITABLE
                                | PageTableFlags::NO_EXECUTE,
                            false,
                            |_| uefi_frame_allocator(st.boot_services())(),
                        )
                        .unwrap();
                },
            );
        }

        info!(
            "Mapped stack to 0x{:X}",
            stack_base.start_address().as_u64()
        );

        stack_top
    };

    let kernel_arguments_box: &'static mut MaybeUninit<KernelArguments> =
        Box::leak(Box::new(MaybeUninit::zeroed()));

    info!("Exiting boot services");

    let st =
        PhysicalMemoryMap::global(|map| exit_boot_services(image, st, map));

    // Activate the new page table
    unsafe {
        page_table.activate();
    };

    assert!(unsafe {
        page_table.mapper().translate_page(stack_top - 1u64).is_ok()
    });
    assert!(unsafe {
        page_table
            .mapper()
            .translate_page(Page::<Size4KiB>::containing_address(
                VirtAddr::from_ptr(kernel.entry.as_ptr::<()>()),
            ))
            .is_ok()
    });

    unsafe {
        let kernel_arguments = kernel_arguments_box.write(KernelArguments {
            st,
            rsdp,
            physical_memory_map: PhysicalMemoryMap::take_global(),
            identity_base: desired_identity_base,
        }) as *mut KernelArguments;
        let kernel_arguments = (VirtAddr::from_ptr(kernel_arguments)
            + desired_identity_base.start_address().as_u64())
        .as_mut_ptr::<KernelArguments>();

        let kernel_entry = core::mem::transmute::<*mut (), KernelEntrySignature>(
            kernel.entry.as_mut_ptr(),
        );

        call_with_stack(
            kernel_arguments,
            kernel_entry,
            stack_top.start_address().as_mut_ptr(),
        );
    };

    exit(-2)
}

pub fn exit(status: i32) -> ! {
    qemu_exit::x86::exit::<u32, { 0xf4 }>(status as u32)
}
