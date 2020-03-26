use crate::physical::{
    allocator::{
        ExternalPhysicalMemoryMapFrameAllocator,
        PhysicalMemoryMapFrameAllocator,
    },
    page_usage::{PageUsage, PageUsageRawType},
};
use ffi_utils::ffi_slice::FfiSliceMut;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{
    frame::PhysFrameRange, FrameDeallocator, PhysFrame, Size4KiB,
    UnusedPhysFrame,
};

static mut PHYSICAL_MEMORY_MAP: Option<Mutex<PhysicalMemoryMap<'static>>> =
    None;

#[repr(C)]
pub struct PhysicalMemoryMap<'buf> {
    buffer: FfiSliceMut<'buf, PageUsageRawType>,
    base: PhysFrame,
}

impl<'buf> PhysicalMemoryMap<'buf> {
    /// Constructs a new memory map
    ///
    /// This will initialize all entries to value.
    ///
    /// In practice, this function is intended to only be called by the bootloader,
    /// which somehow finds out what the physical memory looks like
    /// and passes this on to the os.
    pub fn create(
        buffer: &'buf mut [PageUsageRawType],
        base: PhysFrame,
        value: PageUsage,
    ) -> Self {
        let value = value.to_raw().unwrap();

        for it in buffer.iter_mut() {
            *it = value;
        }

        PhysicalMemoryMap {
            buffer: buffer.into(),
            base,
        }
    }

    /// Creates a memory map from the raw innards
    ///
    /// # Safety
    /// This is only safe if the parts have been obtained by calling release
    /// on a PhysicalMemoryMap instance.
    pub unsafe fn from_raw_parts(
        buffer: &'buf mut [PageUsageRawType],
        base: PhysFrame,
    ) -> Self {
        PhysicalMemoryMap {
            buffer: buffer.into(),
            base,
        }
    }

    /// Consume self and return the underlying buffer
    ///
    /// This does not deallocate the buffer
    #[inline(always)]
    pub fn release(self) -> (&'buf mut [PageUsageRawType], PhysFrame) {
        (self.buffer.into(), self.base)
    }

    pub fn set(
        &mut self,
        frame: PhysFrame,
        value: PageUsage,
    ) -> Option<PageUsage> {
        let value = value.to_raw()?;
        let base = self.base();

        self.buffer_mut()
            .get_mut((frame - base) as usize)
            .map(|r| core::mem::replace(r, value))
            .map(|v| PageUsage::from_raw(v).unwrap())
    }

    pub fn get(&self, frame: PhysFrame) -> Option<PageUsage> {
        self.buffer()
            .get((frame - self.base()) as usize)
            .map(|v| PageUsage::from_raw(*v).unwrap())
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[PageUsageRawType] {
        self.buffer.as_slice()
    }

    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut [PageUsageRawType] {
        self.buffer.as_slice_mut()
    }

    #[inline(always)]
    pub fn pages(&self) -> u64 {
        self.buffer.len() as u64
    }

    #[inline(always)]
    pub fn base(&self) -> PhysFrame {
        self.base
    }

    pub fn physical_range(&self) -> PhysFrameRange {
        PhysFrameRange {
            start: self.base(),
            end: self.base() + self.pages(),
        }
    }

    pub fn iter<'this>(&'this self) -> impl 'this + Iterator<Item = PageUsage> {
        self.buffer()
            .iter()
            .map(|v| PageUsage::from_raw(*v).unwrap())
    }

    pub fn find_unused_frame(&self) -> Option<UnusedPhysFrame> {
        self.iter()
            .enumerate()
            .find(|(_, usage)| usage.is_empty())
            .map(|(index, _)| self.base + (index as u64))
            .map(|frame| unsafe { UnusedPhysFrame::new(frame) })
    }

    pub fn frame_allocator<'this>(
        &'this mut self,
        usage: PageUsage,
    ) -> PhysicalMemoryMapFrameAllocator<'this, 'buf>
    where
        'buf: 'this,
    {
        assert_ne!(usage, PageUsage::Empty);
        assert_ne!(usage, PageUsage::Unusable);

        PhysicalMemoryMapFrameAllocator::new(self, usage)
    }

    /// # Safety
    /// You better trust this external function to know what its doing
    pub unsafe fn external_frame_allocator<'this, A>(
        &'this mut self,
        usage: PageUsage,
        allocator: A,
    ) -> ExternalPhysicalMemoryMapFrameAllocator<'this, 'buf, A>
    where
        'buf: 'this,
        A: FnMut(&PhysicalMemoryMap<'buf>) -> Option<UnusedPhysFrame>,
    {
        assert_ne!(usage, PageUsage::Empty);
        assert_ne!(usage, PageUsage::Unusable);

        ExternalPhysicalMemoryMapFrameAllocator::new(self, usage, allocator)
    }
}

impl PhysicalMemoryMap<'static> {
    /// # Safety
    /// You must initialize PhysicalMemoryMap before any system tries to interact with it.
    /// And if you mess it up, the system will have no idea what is going on.
    /// This is also not thread safe.
    /// In general, once the system is up and running, you should not mess with this.
    pub unsafe fn register_global(self) {
        assert!(PHYSICAL_MEMORY_MAP.is_none());
        PHYSICAL_MEMORY_MAP = Some(Mutex::new(self));
    }

    /// Take the global memory map away.
    ///
    /// # Safety
    /// Any system that needs the physical memory map will no longer work while it is taken away.
    /// In addition, this is not thread safe.
    /// In general, once the system is up and running, you should not mess with this.
    pub unsafe fn take_global() -> Self {
        PHYSICAL_MEMORY_MAP.take().unwrap().into_inner()
    }

    /// Lock the global memory map
    pub fn global() -> MutexGuard<'static, Self> {
        unsafe { PHYSICAL_MEMORY_MAP.as_ref().unwrap().lock() }
    }

    /// Try to lock the global memory map
    ///
    /// This will return None if someone else is holding the lock
    pub fn try_global() -> Option<MutexGuard<'static, Self>> {
        unsafe {
            PHYSICAL_MEMORY_MAP.as_ref().unwrap().try_lock()
        }
    }
}

impl<'buf> FrameDeallocator<Size4KiB> for PhysicalMemoryMap<'buf> {
    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<Size4KiB>) {
        self.set(frame.frame(), PageUsage::Empty);
    }
}
