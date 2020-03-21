#![no_std]

use page_usage::{PageUsage, PhysicalMemoryMap};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        mapper::PhysToVirt, FrameAllocator, MappedPageTable, Mapper, Page,
        PageTable, PageTableFlags, PhysFrame, Size4KiB, UnusedPhysFrame,
    },
    PhysAddr,
};

#[repr(C)]
pub struct IdentityMappedPageTable {
    root: PhysFrame,
    identity_base: Page,
}

fn identity_mapped_phys_to_virt(identity_base: Page) -> impl PhysToVirt {
    move |frame: PhysFrame| {
        (identity_base.start_address() + (frame.start_address()).as_u64())
            .as_mut_ptr()
    }
}

impl IdentityMappedPageTable {
    pub unsafe fn from_raw_parts(root: PhysFrame, identity_base: Page) -> Self {
        IdentityMappedPageTable {
            root,
            identity_base,
        }
    }

    /// Creates a new IdentityMappedPageTable
    ///
    /// This is unsafe because
    /// it wants to write to a bunch of physical pages based on current_identity_base
    #[allow(unused_unsafe)]
    pub unsafe fn create(
        identity_base: Page,

        physical_memory_map: &mut PhysicalMemoryMap,

        current_identity_base: Page,
    ) -> Self {
        // TODO support physical bases other than 0
        let physical_base =
            PhysFrame::from_start_address(PhysAddr::new(0)).unwrap();

        let physical_size = physical_memory_map.pages();

        let current_phys_to_virt =
            identity_mapped_phys_to_virt(current_identity_base);

        fn allocate_page<A, PtV>(
            allocator: &mut A,
            phys_to_virt: &PtV,
        ) -> Option<PhysFrame>
        where
            A: FrameAllocator<Size4KiB>,
            PtV: PhysToVirt,
        {
            allocator.allocate_frame().map(|frame| {
                let frame = frame.frame();
                {
                    let table = phys_to_virt.phys_to_virt(frame);
                    unsafe { table.write(PageTable::new()) };
                }
                frame
            })
        }

        // Allocate the root page
        let root = allocate_page(
            &mut physical_memory_map.frame_allocator(
                PageUsage::PageTableRoot { reference_count: 0 },
            ),
            &current_phys_to_virt,
        )
        .unwrap();

        let level_4_table =
            unsafe { &mut *current_phys_to_virt.phys_to_virt(root) };

        for entry in level_4_table.iter_mut().skip(512 / 2) {
            let page = allocate_page(
                &mut physical_memory_map.frame_allocator(
                    PageUsage::PageTable { reference_count: 0 },
                ),
                &current_phys_to_virt,
            )
            .unwrap();
            entry.set_frame(
                page,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
        }

        // Create a MappedPageTable using the old identity mapping
        let mut intermediate_table = unsafe {
            MappedPageTable::new(level_4_table, current_phys_to_virt)
        };

        // Map all physical pages to their identity position
        let mut allocator = physical_memory_map
            .frame_allocator(PageUsage::PageTable { reference_count: 0 });

        for it in 0..physical_size {
            intermediate_table
                .map_to(
                    identity_base + it,
                    unsafe { UnusedPhysFrame::new(physical_base + it) },
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    &mut allocator,
                )
                .unwrap()
                .ignore();
        }

        IdentityMappedPageTable {
            root,
            identity_base,
        }
    }

    /// Get the page table
    ///
    /// This is unsafe because it only works correctly if self is the active page table
    pub unsafe fn get_page_table_mut(
        &mut self,
    ) -> MappedPageTable<impl PhysToVirt> {
        self.get_page_table_mut_in_different_identity_mapping(
            self.identity_base,
        )
    }

    /// Get the page table
    ///
    /// This is unsafe because it only works correctly if
    /// identity_base has a full mapping of all physical pages
    pub unsafe fn get_page_table_mut_in_different_identity_mapping(
        &mut self,
        identity_base: Page,
    ) -> MappedPageTable<impl PhysToVirt> {
        let phys_to_virt = identity_mapped_phys_to_virt(identity_base);

        let root = phys_to_virt.phys_to_virt(self.root);

        MappedPageTable::new(&mut *root, phys_to_virt)
    }

    pub unsafe fn activate(&self) {
        Cr3::write(self.root, Cr3Flags::empty());
    }
}
