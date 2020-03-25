use crate::traits::{Allocator, OwnerCheck};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    ptr::NonNull,
};
use spin::Mutex;

/// Create an AllocRef from a guarded Allocator pointer
#[derive(Copy, Clone)]
pub struct MagicAllocRef<A>
where
    A: Allocator,
{
    allocator: NonNull<Mutex<A>>,
}

impl<A> MagicAllocRef<A>
where
    A: Allocator,
{
    pub unsafe fn new(allocator: NonNull<Mutex<A>>) -> Self {
        MagicAllocRef { allocator }
    }

    pub unsafe fn allocator<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut A) -> R,
    {
        let mut lock = self.allocator.as_ref().lock();
        function(&mut lock)
    }
}

unsafe impl<A> AllocRef for MagicAllocRef<A>
where
    A: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        unsafe { self.allocator(|a| a.alloc(layout)) }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.allocator(|a| a.dealloc(ptr, layout))
    }
}

impl<A> OwnerCheck for MagicAllocRef<A>
where
    A: Allocator + OwnerCheck,
{
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        unsafe { self.allocator(|a| a.is_owner(ptr, layout)) }
    }
}
