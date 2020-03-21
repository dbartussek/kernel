#![no_std]

mod page_usage;

pub use self::page_usage::*;
use ffi_utils::ffi_slice::FfiSliceMut;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB, UnusedPhysFrame,
};

#[repr(C)]
pub struct PhysicalMemoryMap<'buf> {
    buffer: FfiSliceMut<'buf, PageUsageRawType>,
    base: PhysFrame,
}

impl<'buf> PhysicalMemoryMap<'buf> {
    pub fn new(
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

    /// Consume self and return the underlying buffer
    ///
    /// This does not deallocate the buffer
    #[inline(always)]
    pub fn release_buffer(self) -> &'buf mut [PageUsageRawType] {
        self.buffer.into()
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

        PhysicalMemoryMapFrameAllocator { map: self, usage }
    }
}

pub struct PhysicalMemoryMapFrameAllocator<'map, 'buf>
where
    'buf: 'map,
{
    map: &'map mut PhysicalMemoryMap<'buf>,
    usage: PageUsage,
}

unsafe impl<'map, 'buf> FrameAllocator<Size4KiB>
    for PhysicalMemoryMapFrameAllocator<'map, 'buf>
where
    'buf: 'map,
{
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        self.map.find_unused_frame().map(|frame| {
            self.map.set((&frame as &PhysFrame).clone(), self.usage);
            frame
        })
    }
}

impl<'buf> FrameDeallocator<Size4KiB> for PhysicalMemoryMap<'buf> {
    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<Size4KiB>) {
        self.set(frame.frame(), PageUsage::Empty);
    }
}
