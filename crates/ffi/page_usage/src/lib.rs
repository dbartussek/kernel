#![no_std]

pub mod allocator;
mod page_usage;

pub use self::page_usage::*;
use crate::allocator::{
    ExternalPhysicalMemoryMapFrameAllocator, PhysicalMemoryMapFrameAllocator,
};
use ffi_utils::ffi_slice::FfiSliceMut;
use x86_64::structures::paging::{
    frame::PhysFrameRange, FrameDeallocator, PhysFrame, Size4KiB,
    UnusedPhysFrame,
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

    pub unsafe fn from_raw_parts(
        buffer: &'buf mut [PageUsageRawType],
        base: PhysFrame,
    ) -> Self {
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
    pub fn release(self) -> (&'buf mut [PageUsageRawType], PhysFrame) {
        (self.buffer.into(), self.base)
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

    /// This is unsafe, because you better trust this external function to know what its doing
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

impl<'buf> FrameDeallocator<Size4KiB> for PhysicalMemoryMap<'buf> {
    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<Size4KiB>) {
        self.set(frame.frame(), PageUsage::Empty);
    }
}
