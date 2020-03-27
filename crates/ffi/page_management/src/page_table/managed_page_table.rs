use crate::{
    page_table::{identity_base, identity_page},
    physical::{map::PhysicalMemoryMap, page_usage::PageUsage},
};
use log::*;
use spin::{Mutex, MutexGuard};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, OffsetPageTable, Page,
        PageTable, PageTableFlags, PhysFrame, Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

/// The kernel address space starts at this address (start of canonical high half)
pub const KERNEL_ADDRESS_SPACE_BASE: u64 = 0xffff_8000_0000_0000;

/// The user space goes from 0 to this - 1
pub const USER_ADDRESS_SPACE_END: u64 = 0x0000_8000_0000_0000;

/// Size of each kernel region in bytes
pub const KERNEL_REGION_SIZE: u64 = 0x1000_0000_0000;

const IDENTITY_REGION: u64 = 0;
const IDENTITY_SIZE: u64 = 1;

#[allow(clippy::erasing_op)]
pub const IDENTITY_BASE: u64 =
    KERNEL_ADDRESS_SPACE_BASE + KERNEL_REGION_SIZE * IDENTITY_REGION;
#[allow(clippy::identity_op, dead_code)]
const IDENTITY_END: u64 = KERNEL_ADDRESS_SPACE_BASE
    + KERNEL_REGION_SIZE * (IDENTITY_REGION + IDENTITY_SIZE);

const KERNEL_HEAP_REGION: u64 = 6;
pub const KERNEL_HEAP_BASE: u64 =
    KERNEL_ADDRESS_SPACE_BASE + KERNEL_REGION_SIZE * KERNEL_HEAP_REGION;

const KERNEL_STACK_REGION: u64 = 7;
pub const KERNEL_STACK_BASE: u64 =
    KERNEL_ADDRESS_SPACE_BASE + KERNEL_REGION_SIZE * KERNEL_STACK_REGION;

fn address_region(address: VirtAddr) -> u64 {
    if !is_in_kernel_space(address) {
        return core::u64::MAX;
    }

    let raw =
        (address.as_u64() - KERNEL_ADDRESS_SPACE_BASE) / KERNEL_REGION_SIZE;

    if raw >= IDENTITY_REGION && raw <= (IDENTITY_REGION + IDENTITY_SIZE) {
        IDENTITY_REGION
    } else {
        raw
    }
}

pub fn is_in_user_space(address: VirtAddr) -> bool {
    address.as_u64() < USER_ADDRESS_SPACE_END
}
pub fn is_in_kernel_space(address: VirtAddr) -> bool {
    address.as_u64() >= KERNEL_ADDRESS_SPACE_BASE
}

/// A standard page table
///
/// All page tables share their high half mappings and have unique user space mappings.
#[repr(transparent)]
pub struct ManagedPageTable {
    root: u64,
}

impl ManagedPageTable {
    /// # Safety
    /// This creates a KernelPageTable from an arbitrary Physical Frame.
    /// This is only valid if a proper page table has been constructed there
    pub unsafe fn from_raw_frame(frame: PhysFrame<Size4KiB>) -> Self {
        ManagedPageTable {
            root: frame.start_address().as_u64(),
        }
    }

    /// Read the currently active page table
    ///
    /// # Safety
    /// You don't know which one this exactly is. You should keep your hands off of any
    /// unserspace mappings
    pub unsafe fn read_global() -> Self {
        Self::from_raw_frame(Cr3::read().0)
    }

    /// Get the raw, underlying physical frame
    ///
    /// # Safety
    /// You should prefer using self.modify to make changes.
    /// Using this function, you can create copies of the page table, which may lead to
    /// double frees of page tables.
    pub unsafe fn frame(&self) -> PhysFrame<Size4KiB> {
        PhysFrame::from_start_address(PhysAddr::new(self.root)).unwrap()
    }

    /// Write this page table to Cr3
    ///
    /// # Safety
    /// You can break any and all pointers by using this.
    /// You better be sure all old references are still valid after changing the page table
    pub unsafe fn activate(&self) -> PhysFrame<Size4KiB> {
        let (old_frame, _flags) = Cr3::read();
        Cr3::write(self.frame(), Cr3Flags::empty());
        old_frame
    }

    pub unsafe fn page_table_ref(&self) -> &PageTable {
        &*identity_page(self.frame()).start_address().as_ptr()
    }

    pub unsafe fn page_table_mut(&mut self) -> &mut PageTable {
        &mut *identity_page(self.frame()).start_address().as_mut_ptr()
    }

    pub unsafe fn mapper(&mut self) -> OffsetPageTable {
        OffsetPageTable::new(
            self.page_table_mut(),
            identity_base().start_address(),
        )
    }

    pub fn create_offspring(&self) -> Option<Self> {
        let mut physical_memory_map = PhysicalMemoryMap::global();
        let root_frame = physical_memory_map
            .frame_allocator(PageUsage::PageTableRoot)
            .allocate_frame()?
            .frame();

        unsafe {
            let mut child = ManagedPageTable::from_raw_frame(root_frame);

            let self_table = self.page_table_ref();
            let child_table = child.page_table_mut();

            child_table.zero();
            let half_size = self_table.iter().count() / 2;

            for (child_it, self_it) in child_table
                .iter_mut()
                .zip(self_table.iter())
                .skip(half_size)
            {
                *child_it = self_it.clone();
            }

            Some(child)
        }
    }

    /// Tears down the page table and releases all memory used for user space mappings.
    ///
    /// # Safety
    /// The memory used for this page table is released.
    /// Any attempts to use it at a later point will lead to nasty bugs.
    pub unsafe fn dispose(self) {
        assert_ne!(self.frame(), Cr3::read().0);
        unimplemented!()
    }

    pub fn modify(&mut self, flags: ModificationFlags) -> ModificationManager {
        ModificationManager {
            guards: MUTEXES.lock(flags),
            page_table: self,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct ModificationFlags {
    pub identity: bool,
    pub kernel_stack: bool,
    pub kernel_heap: bool,
}

struct ModificationMutexes {
    identity: Mutex<()>,
    kernel_stack: Mutex<()>,
    kernel_heap: Mutex<()>,
}

impl ModificationMutexes {
    pub fn lock(&self, flags: ModificationFlags) -> ModificationGuards {
        let identity = if flags.identity {
            Some(self.identity.lock())
        } else {
            None
        };
        let kernel_stack = if flags.kernel_stack {
            Some(self.kernel_stack.lock())
        } else {
            None
        };
        let kernel_heap = if flags.kernel_heap {
            Some(self.kernel_heap.lock())
        } else {
            None
        };

        ModificationGuards {
            identity,
            kernel_stack,
            kernel_heap,
        }
    }
}

struct ModificationGuards<'lt> {
    identity: Option<MutexGuard<'lt, ()>>,
    kernel_stack: Option<MutexGuard<'lt, ()>>,
    kernel_heap: Option<MutexGuard<'lt, ()>>,
}

static MUTEXES: ModificationMutexes = ModificationMutexes {
    identity: Mutex::new(()),
    kernel_stack: Mutex::new(()),
    kernel_heap: Mutex::new(()),
};

/// A struct that makes sure the correct Mutexes are held to make the modifications safe(ish)
pub struct ModificationManager<'page_table> {
    guards: ModificationGuards<'static>,
    page_table: &'page_table mut ManagedPageTable,
}

impl<'page_table> ModificationManager<'page_table> {
    unsafe fn map_pages_impl<It, A>(
        &mut self,
        start_page: Page<Size4KiB>,
        frames: It,
        flags: PageTableFlags,
        flush: bool,
        mut frame_allocator: A,
    ) -> Result<(), ()>
    where
        It: ExactSizeIterator + Iterator<Item = PhysFrame>,
        A: FrameAllocator<Size4KiB>,
    {
        let frame_count = frames.len() as u64;

        self.is_valid_range(PageRange {
            start: start_page,
            end: start_page + frame_count,
        })?;

        let mut mapper = self.page_table.mapper();

        for (index, frame) in frames
            .enumerate()
            .map(|(index, frame)| (index as u64, frame))
        {
            let flusher = mapper
                .map_to(
                    start_page + index,
                    UnusedPhysFrame::new(frame),
                    flags,
                    &mut frame_allocator,
                )
                .unwrap();
            if flush {
                flusher.flush();
            } else {
                flusher.ignore();
            }
        }

        Ok(())
    }

    pub unsafe fn map_pages<It>(
        &mut self,
        start_page: Page<Size4KiB>,
        frames: It,
        flags: PageTableFlags,
        flush: bool,
    ) -> Result<(), ()>
    where
        It: ExactSizeIterator + Iterator<Item = PhysFrame>,
    {
        let mut physical_map = PhysicalMemoryMap::global();
        let allocator = physical_map.frame_allocator(PageUsage::PageTable);

        self.map_pages_impl(start_page, frames, flags, flush, allocator)
    }

    pub unsafe fn map_pages_external_frame_allocator<It, A>(
        &mut self,
        start_page: Page<Size4KiB>,
        frames: It,
        flags: PageTableFlags,
        flush: bool,
        allocate: A,
    ) -> Result<(), ()>
    where
        It: ExactSizeIterator + Iterator<Item = PhysFrame>,
        A: FnMut(&PhysicalMemoryMap) -> Option<UnusedPhysFrame>,
    {
        let mut physical_map = PhysicalMemoryMap::global();
        let allocator = physical_map
            .external_frame_allocator(PageUsage::PageTable, allocate);

        self.map_pages_impl(start_page, frames, flags, flush, allocator)
    }

    fn is_valid_range(&self, range: PageRange<Size4KiB>) -> Result<(), ()> {
        info!("{:X?}", &range);

        let PageRange { start, end } = range;

        info!("Size: 0x{:X} pages", end - start);

        let start = start.start_address();
        let end = end.start_address();

        let start_region = address_region(start);
        let end_region = address_region(end - 1u64);

        // Regions must be either in user space or in kernel space
        if is_in_user_space(start) != is_in_user_space(end) {
            error!("Desired page range crosses user space boundary");
            return Err(());
        }
        if is_in_kernel_space(start) != is_in_kernel_space(end) {
            error!("Desired page range crosses kernel space boundary");
            return Err(());
        }

        if !is_in_user_space(start) && !is_in_kernel_space(start) {
            error!("Desired page range is in non-canonical range");
            return Err(());
        }

        if is_in_kernel_space(start) {
            if start_region != end_region {
                error!(
                    "Desired page range spans kernal space regions ({}, {})",
                    start_region, end_region
                );
                return Err(());
            }

            // Make sure the lock is held for modifications in the region
            match start_region {
                IDENTITY_REGION => {
                    if self.guards.identity.is_none() {
                        error!(
                            "Attempted to modify identity region without lock"
                        );
                        return Err(());
                    }
                },
                KERNEL_HEAP_REGION => {
                    if self.guards.kernel_heap.is_none() {
                        error!("Attempted to modify kernel heap without lock");
                        return Err(());
                    }
                },
                KERNEL_STACK_REGION => {
                    if self.guards.kernel_stack.is_none() {
                        error!("Attempted to modify kernel stack without lock");
                        return Err(());
                    }
                },
                _ => {
                    error!(
                        "Attempted to modify unknown region {}",
                        start_region
                    );
                    return Err(());
                },
            }
        }

        Ok(())
    }
}
