use crate::{
    page_table::{identity_base, manager::KernelPageTableManager},
    physical::{map::PhysicalMemoryMap, page_usage::PageUsage},
};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        mapper::PhysToVirt, FrameAllocator, MappedPageTable, Mapper,
        OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
        UnusedPhysFrame,
    },
    PhysAddr,
};

#[repr(transparent)]
pub struct KernelPageTable {
    root: u64,
}

pub fn identity_mapped_phys_to_virt(identity_base: Page) -> impl PhysToVirt {
    move |frame: PhysFrame| {
        (identity_base.start_address() + (frame.start_address()).as_u64())
            .as_mut_ptr()
    }
}

impl KernelPageTable {
    pub unsafe fn from_raw_parts(root: PhysFrame) -> Self {
        KernelPageTable {
            root: root.start_address().as_u64(),
        }
    }

    pub fn current_page_table() -> Self {
        unsafe { Self::from_raw_parts(Cr3::read().0) }
    }

    fn root(&self) -> PhysFrame {
        PhysFrame::from_start_address(PhysAddr::new(self.root)).unwrap()
    }

    #[allow(unused_unsafe)]
    pub unsafe fn initialize_and_create<A>(
        physical_memory_map: &mut PhysicalMemoryMap,
        current_identity_base: Page,
        mut allocate: A,
    ) -> Self
    where
        A: FnMut(&PhysicalMemoryMap) -> Option<UnusedPhysFrame>,
    {
        unsafe {
            crate::page_table::initialize(current_identity_base);
        }

        // TODO support physical bases other than 0
        let physical_base = physical_memory_map.base();
        assert_eq!(physical_base.start_address().as_u64(), 0);

        let physical_range = physical_memory_map.physical_range();

        let current_phys_to_virt =
            identity_mapped_phys_to_virt(current_identity_base);

        fn create_page_table<A, PtV>(
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
        let root = create_page_table(
            &mut physical_memory_map.external_frame_allocator(
                PageUsage::PageTableRoot { reference_count: 0 },
                &mut allocate,
            ),
            &current_phys_to_virt,
        )
        .unwrap();

        let level_4_table =
            unsafe { &mut *current_phys_to_virt.phys_to_virt(root) };

        // Fill the kernel space top level entries
        for entry in level_4_table.iter_mut().skip(512 / 2) {
            let page = create_page_table(
                &mut physical_memory_map.external_frame_allocator(
                    PageUsage::PageTable { reference_count: 0 },
                    &mut allocate,
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
        let mut manager = KernelPageTableManager::new(unsafe {
            OffsetPageTable::new(
                level_4_table,
                current_identity_base.start_address(),
            )
        });

        // Map all physical pages to their identity position
        let mut allocator = physical_memory_map.external_frame_allocator(
            PageUsage::PageTable { reference_count: 0 },
            &mut allocate,
        );

        manager.map_range(
            physical_range,
            current_identity_base,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            false,
            &mut allocator,
        );

        KernelPageTable::from_raw_parts(root)
    }

    /// Get the page table
    ///
    /// # Safety
    /// This is unsafe because it allows for arbitrary modification of the page table.
    pub unsafe fn get_page_table_mut(
        &mut self,
    ) -> MappedPageTable<impl PhysToVirt> {
        self.get_page_table_mut_in_different_identity_mapping(identity_base())
    }

    /// Get the page table
    ///
    /// # Safety
    /// This is unsafe because it only works correctly if
    /// identity_base has a full mapping of all physical pages.
    /// It also allows for arbitrary modification of the page table
    pub unsafe fn get_page_table_mut_in_different_identity_mapping(
        &mut self,
        identity_base: Page,
    ) -> MappedPageTable<impl PhysToVirt> {
        let phys_to_virt = identity_mapped_phys_to_virt(identity_base);

        let root = phys_to_virt.phys_to_virt(self.root());

        MappedPageTable::new(&mut *root, phys_to_virt)
    }

    pub fn get_manager<'this>(
        &'this mut self,
    ) -> KernelPageTableManager<impl 'this + Mapper<Size4KiB>, Size4KiB> {
        KernelPageTableManager::new(unsafe { self.get_page_table_mut() })
    }

    /// Write this page table to Cr3
    ///
    /// # Safety
    /// You can break any and all pointers by using this.
    /// You better be sure all old references are still valid after changing the page table
    pub unsafe fn activate(&self) {
        Cr3::write(self.root(), Cr3Flags::empty());
    }
}
