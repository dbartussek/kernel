use crate::{PageUsage, PhysicalMemoryMap};
use x86_64::structures::paging::{
    FrameAllocator, PhysFrame, Size4KiB, UnusedPhysFrame,
};

pub struct PhysicalMemoryMapFrameAllocator<'map, 'buf>
where
    'buf: 'map,
{
    map: &'map mut PhysicalMemoryMap<'buf>,
    usage: PageUsage,
}

impl<'map, 'buf> PhysicalMemoryMapFrameAllocator<'map, 'buf>
where
    'buf: 'map,
{
    pub fn new(
        map: &'map mut PhysicalMemoryMap<'buf>,
        usage: PageUsage,
    ) -> Self {
        PhysicalMemoryMapFrameAllocator { map, usage }
    }
}

unsafe impl<'map, 'buf> FrameAllocator<Size4KiB>
    for PhysicalMemoryMapFrameAllocator<'map, 'buf>
where
    'buf: 'map,
{
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        unsafe {
            ExternalPhysicalMemoryMapFrameAllocator::new(
                &mut self.map,
                self.usage,
                |map| map.find_unused_frame(),
            )
        }
        .allocate_frame()
    }
}

pub struct ExternalPhysicalMemoryMapFrameAllocator<'map, 'buf, A>
where
    'buf: 'map,
    A: FnMut(&PhysicalMemoryMap<'buf>) -> Option<UnusedPhysFrame>,
{
    map: &'map mut PhysicalMemoryMap<'buf>,
    usage: PageUsage,
    allocator: A,
}

impl<'map, 'buf, A> ExternalPhysicalMemoryMapFrameAllocator<'map, 'buf, A>
where
    'buf: 'map,
    A: FnMut(&PhysicalMemoryMap<'buf>) -> Option<UnusedPhysFrame>,
{
    /// Create a new FrameAllocator with external decision making
    ///
    /// This is unsafe, because you better trust this external function to know what its doing
    pub unsafe fn new(
        map: &'map mut PhysicalMemoryMap<'buf>,
        usage: PageUsage,
        allocator: A,
    ) -> Self {
        ExternalPhysicalMemoryMapFrameAllocator {
            map,
            usage,
            allocator,
        }
    }
}

unsafe impl<'map, 'buf, A> FrameAllocator<Size4KiB>
    for ExternalPhysicalMemoryMapFrameAllocator<'map, 'buf, A>
where
    'buf: 'map,
    A: FnMut(&PhysicalMemoryMap<'buf>) -> Option<UnusedPhysFrame>,
{
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        (self.allocator)(&self.map).map(|frame| {
            self.map.set((&frame as &PhysFrame).clone(), self.usage);
            frame
        })
    }
}
