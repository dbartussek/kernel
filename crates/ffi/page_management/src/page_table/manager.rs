use core::{fmt::Debug, marker::PhantomData};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageSize, PageTableFlags, PhysFrame,
    Size4KiB, UnusedPhysFrame,
};

pub struct KernelPageTableManager<M, S>
where
    S: PageSize + Debug,
    M: Mapper<S>,
{
    mapper: M,
    _size: PhantomData<S>,
}

impl<M, S> KernelPageTableManager<M, S>
where
    S: PageSize + Debug,
    M: Mapper<S>,
{
    pub fn new(mapper: M) -> Self {
        KernelPageTableManager {
            mapper,
            _size: Default::default(),
        }
    }

    pub fn mapper(&self) -> &M {
        &self.mapper
    }

    /// Access the underlying mapper
    ///
    /// You can break invariants this way, so be careful
    pub unsafe fn mapper_mut(&mut self) -> &mut M {
        &mut self.mapper
    }

    pub fn map_range<It, A>(
        &mut self,
        range: It,
        base: Page<S>,
        flags: PageTableFlags,
        flush: bool,
        a: &mut A,
    ) where
        It: IntoIterator<Item = PhysFrame<S>>,
        A: FrameAllocator<Size4KiB>,
    {
        for (index, frame) in range.into_iter().enumerate() {
            let index = index as u64;

            let flusher = self
                .mapper
                .map_to(
                    base + index,
                    unsafe { UnusedPhysFrame::new(frame) },
                    flags,
                    a,
                )
                .unwrap();
            if flush {
                flusher.flush();
            } else {
                flusher.ignore();
            }
        }
    }
}
